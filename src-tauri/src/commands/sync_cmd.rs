//! Cloud Sync Commands
//!
//! Tauri commands for managing cloud synchronization with Turso.

use crate::repository::{configure_sync, get_sync_config, SyncConfig};
use std::path::PathBuf;
use tauri::Manager;

/// Get database path from app handle
fn get_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let app_dir = app_handle.path().app_data_dir().unwrap();
    std::fs::create_dir_all(&app_dir).unwrap();
    app_dir.join("tag_all.db")
}

/// Configure cloud sync with Turso database (with data backup and migration)
#[tauri::command]
pub async fn configure_cloud_sync(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    url: String,
    token: String,
) -> Result<(), String> {
    use crate::repository;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use std::time::Duration;
    
    eprintln!("=== Cloud Sync Configuration Start ===");
    eprintln!("URL: {}, Token len: {}", url, token.len());
    
    // Validate connection first
    if !url.is_empty() && !token.is_empty() {
        crate::repository::db::validate_cloud_connection(url.clone(), token.clone()).await
            .map_err(|e| format!("验证连接失败: {}", e))?;
    }
    
    let db_path = get_db_path(&app_handle);
    let backup_json_path = db_path.with_extension("db.backup.json");
    let safety_backup_path = db_path.with_extension("db.safety_backup");
    
    // === STEP 1: Backup existing data ===
    eprintln!("[1/7] Backing up existing data...");
    let backup_data = {
        match state.db_state.get_connection().await {
            Ok(old_conn) => {
                match repository::db::backup_local_data(&old_conn).await {
                    Ok(backup) => {
                        eprintln!("✓ Memory backup: {} items, {} tags, {} workspaces",
                                  backup.items.len(), backup.tags.len(), backup.workspaces.len());
                        
                        // Save to JSON file as extra insurance
                        if let Ok(json_str) = serde_json::to_string(&backup) {
                            let _ = std::fs::write(&backup_json_path, json_str);
                            eprintln!("✓ JSON backup saved");
                        }
                        
                        Some(backup)
                    }
                    Err(e) => {
                        eprintln!("⚠ Backup failed: {}, continuing without backup", e);
                        None
                    }
                }
            }
            Err(_) => {
                eprintln!("⚠ No existing connection, skipping backup");
                None
            }
        }
    };
    
    // === STEP 2: Close existing connections ===
    eprintln!("[2/7] Closing existing connections...");
    state.db_state.close().await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    eprintln!("✓ Connections closed");
    
    //=== STEP 3: Create safety backup of database file ===
    eprintln!("[3/7] Creating safety backup...");
    if db_path.exists() {
        match std::fs::copy(&db_path, &safety_backup_path) {
            Ok(_) => eprintln!("✓ Safety backup created"),
            Err(e) => eprintln!("⚠ Safety backup failed: {}", e),
        }
    }

    // === STEP 3.5: Migrate remote schema ===
    eprintln!("[3.5/7] Migrating remote schema...");
    if let Err(e) = repository::db::migrate_remote_schema(url.clone(), token.clone()).await {
        eprintln!("⚠ Remote migration warning: {}", e);
        // We continue because maybe it's connection issue but sync might still work or schema is already good
    } else {
        eprintln!("✓ Remote schema migrated");
    }
    
    // === STEP 4: Save configuration ===
    eprintln!("[4/7] Saving sync configuration...");
    repository::configure_sync(&db_path, url.clone(), token.clone()).await?;
    
    // === STEP 5: Delete old DB files to force fresh cloud sync ===
    eprintln!("[5/7] Cleaning old database files...");
    if db_path.exists() {
        let _ = std::fs::remove_file(&db_path);
    }
    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
    
    // Remove sync metadata
    let sync_dir = db_path.parent().unwrap()
        .join(format!("{}-sync", db_path.file_name().unwrap().to_str().unwrap()));
    if sync_dir.exists() {
        let _ = std::fs::remove_dir_all(&sync_dir);
    }
    
    // === STEP 6: Initialize cloud database ===
    eprintln!("[6/7] Initializing cloud database...");
    match repository::init_db(&db_path).await {
        Ok(new_db_state) => {
            eprintln!("✓ Cloud database initialized");
            
            // Restore data if we have backup
            if let Some(backup) = backup_data {
                eprintln!("Migrating local data to cloud...");
                match new_db_state.get_connection().await {
                    Ok(new_conn) => {
                        match repository::db::restore_data(&new_conn, backup).await {
                            Ok(_) => eprintln!("✓ Data migrated successfully"),
                            Err(e) => eprintln!("⚠ Data migration failed: {}", e),
                        }
                    }
                    Err(e) => eprintln!("⚠ Failed to get connection for restore: {}", e),
                }
            }
            
            // === STEP 7: Update application state ===
            eprintln!("[7/7] Updating application state...");
            let new_conn = new_db_state.get_connection().await?;
            let new_conn = Arc::new(Mutex::new(new_conn));
            
            {
                *state.item_repo.lock().await = repository::ItemRepository::new(new_conn.clone());
                *state.tag_repo.lock().await = repository::TagRepository::new(new_conn.clone());
                *state.window_repo.lock().await = repository::WindowStateRepository::new(new_conn.clone());
                *state.workspace_repo.lock().await = repository::WorkspaceRepository::new(new_conn);
            }
            
            state.db_state.replace_with(&new_db_state).await;
            eprintln!("✓ Application state updated");
            
            // Trigger initial sync
            eprintln!("Triggering initial sync...");
            match new_db_state.sync().await {
                Ok(_) => {
                    eprintln!("✓ Initial sync complete");
                    
                    // Clean up backup files on success
                    let _ = std::fs::remove_file(&backup_json_path);
                    let _ = std::fs::remove_file(&safety_backup_path);
                    eprintln!("=== Cloud Sync Configuration Complete ===");
                    
                    Ok(())
                }
                Err(e) => {
                    eprintln!("✗ Initial sync failed: {}", e);
                    Err(format!("同步失败: {}", e))
                }
            }
        }
        Err(e) => {
            // === ROLLBACK: Cloud init failed ===
            eprintln!("✗ Cloud database initialization failed: {}", e);
            eprintln!("=== ROLLING BACK ===");
            
            // Restore safety backup
            if safety_backup_path.exists() {
                match std::fs::copy(&safety_backup_path, &db_path) {
                    Ok(_) => eprintln!("✓ Database file restored from safety backup"),
                    Err(e) => eprintln!("✗ Failed to restore database: {}", e),
                }
            }
            
            // Delete sync configuration
            let config_path = db_path.parent().unwrap().join("sync_config.json");
            let _ = std::fs::remove_file(&config_path);
            eprintln!("✓ Sync configuration deleted");
            
            // Reinitialize local database
            eprintln!("Reinitializing local database...");
            match repository::init_db(&db_path).await {
                Ok(local_state) => {
                    state.db_state.replace_with(&local_state).await;
                    eprintln!("✓ Local database restored");
                }
                Err(e) => eprintln!("✗ Failed to reinit local database: {}", e),
            }
            
            eprintln!("=== Rollback Complete ===");
            Err(format!("云同步配置失败已回滚: {}", e))
        }
    }
}

/// Get current cloud sync configuration
#[tauri::command]
pub fn get_cloud_sync_config(
    app_handle: tauri::AppHandle,
) -> Result<Option<SyncConfig>, String> {
    let db_path = get_db_path(&app_handle);
    Ok(get_sync_config(&db_path))
}

/// Save cloud sync configuration without triggering sync
#[tauri::command]
pub async fn save_cloud_sync_config(
    app_handle: tauri::AppHandle,
    url: String,
    token: String,
) -> Result<(), String> {
    // Validate connection before saving
    if !url.is_empty() && !token.is_empty() {
        crate::repository::db::validate_cloud_connection(url.clone(), token.clone()).await
            .map_err(|e| format!("验证连接失败: {}", e))?;
    }

    let db_path = get_db_path(&app_handle);
    configure_sync(&db_path, url, token).await
}

/// Manually trigger cloud database sync
#[tauri::command]
pub async fn sync_cloud_db(
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    state.db_state.sync().await
}

/// Check if cloud sync is currently enabled for this session
#[tauri::command]
pub async fn is_cloud_sync_enabled(
    state: tauri::State<'_, crate::AppState>,
) -> Result<bool, String> {
    Ok(state.db_state.is_cloud_sync_enabled().await)
}
