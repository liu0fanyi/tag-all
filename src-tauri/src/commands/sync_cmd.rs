//! Cloud Sync Commands
//!
//! Tauri commands for managing cloud synchronization with Turso.

use crate::repository::{configure_sync as repo_configure_sync, get_sync_config as repo_get_sync_config, SyncConfig};
use std::path::PathBuf;
use tauri::Manager;

// Import validate_cloud_connection from backend crate
use tauri_sync_db_backend::validate_cloud_connection;

// Import generic sync
use tauri_sync_db_backend::sync::{sync_all, DynamicSchema};
use tauri_plugin_http::reqwest;

/// Helper to perform sync using generic backend
async fn perform_sync(state: &tauri_sync_db_backend::DbState) -> Result<(), String> {
    let config = repo_get_sync_config(&state.db_path).ok_or("Sync not configured")?;
    
    // 1. Load Dynamic Schema from DB
    // We want to sync: items, tags, item_tags, tag_tags, workspaces, workspace_dirs, window_state
    // Settings? tag-all doesn't seem to have settings table yet, or it's implicitly handled.
    // Based on db.rs migrations:
    let tables = vec![
        "workspaces", 
        "workspace_dirs", 
        "tags", 
        "items", 
        "item_tags", 
        "tag_tags", 
        "window_state"
    ];
    
    let schema = DynamicSchema::load(state, tables).await
        .map_err(|e| format!("Failed to load schema: {}", e))?;
        
    // 2. HTTP Client
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // For local testing/emulator
        .build()
        .map_err(|e| e.to_string())?;
        
    // 3. Sync All
    sync_all(&client, state, &schema, &config.url, &config.token).await
}

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
    use std::time::Duration;
    
    eprintln!("=== Cloud Sync Configuration Start ===");
    eprintln!("URL: {}, Token len: {}", url, token.len());
    
    // Validate connection first (using shared crate)
    if !url.is_empty() && !token.is_empty() {
        validate_cloud_connection(url.clone(), token.clone()).await
            .map_err(|e| format!("验证连接失败: {}", e))?;
    }
    
    let db_path = get_db_path(&app_handle);
    let backup_json_path = db_path.with_extension("db.backup.json");
    let safety_backup_path = db_path.with_extension("db.safety_backup");
    
    // === STEP 1: Backup existing data ===
    eprintln!("[1/7] Backing up existing data...");
    let backup_data = {
        let conn_opt_guard = state.db_state.conn.lock().await;
        if let Some(old_conn) = conn_opt_guard.as_ref() {
            // Synchronous call - safe because it's just rusqlite on local thread buffer
            match repository::db::backup_local_data(old_conn) {
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
        } else {
            eprintln!("⚠ No existing connection, skipping backup");
            None
        }
    };
    
    // === STEP 2: Close existing connections ===
    eprintln!("[2/7] Closing existing connections...");
    state.db_state.conn.lock().await.take(); // Manually take connection to drop it
    
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

    // === STEP 4: Save configuration ===
    eprintln!("[4/7] Saving sync configuration...");
    repo_configure_sync(&db_path, url.clone(), token.clone()).await?;
    
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
                let conn_guard = new_db_state.conn.lock().await;
                if let Some(new_conn) = conn_guard.as_ref() {
                     // Synchronous call
                     match repository::db::restore_data(new_conn, backup) {
                        Ok(_) => eprintln!("✓ Data migrated successfully"),
                        Err(e) => eprintln!("⚠ Data migration failed: {}", e),
                    }
                } else {
                    eprintln!("⚠ Failed to get connection for restore");
                }
            }
            
            // === STEP 7: Update application state ===
            eprintln!("[7/7] Updating application state...");
            
            // Update DbState
            {
                let mut app_conn = state.db_state.conn.lock().await;
                let new_conn_opt = new_db_state.conn.lock().await.take();
                *app_conn = new_conn_opt;
            }
            
            eprintln!("✓ Application state updated");
            
            // Trigger initial sync
            eprintln!("Triggering initial sync...");
            // Use local helper
            match perform_sync(&state.db_state).await {
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
                    // Restore connection
                     let mut app_conn = state.db_state.conn.lock().await;
                     let new_conn_opt = local_state.conn.lock().await.take();
                     *app_conn = new_conn_opt;
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
    Ok(repo_get_sync_config(&db_path))
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
        validate_cloud_connection(url.clone(), token.clone()).await
            .map_err(|e| format!("验证连接失败: {}", e))?;
    }

    let db_path = get_db_path(&app_handle);
    repo_configure_sync(&db_path, url, token).await
}

/// Manually trigger cloud database sync
#[tauri::command]
pub async fn sync_cloud_db(
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    perform_sync(&state.db_state).await
}

/// Check if cloud sync is currently enabled for this session
#[tauri::command]
pub async fn is_cloud_sync_enabled(
    state: tauri::State<'_, crate::AppState>,
) -> Result<bool, String> {
    Ok(state.db_state.is_cloud_sync_enabled())
}

/// Alias for save_cloud_sync_config (for compatibility with SyncSettingsForm)
#[tauri::command]
pub async fn configure_sync(
    app_handle: tauri::AppHandle,
    url: String,
    token: String,
) -> Result<(), String> {
    save_cloud_sync_config(app_handle, url, token).await
}

/// Alias for sync_cloud_db (for compatibility with SyncButton)
#[tauri::command]
pub async fn sync_database(
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    sync_cloud_db(state).await
}

/// Alias for get_cloud_sync_config (for compatibility with SyncSettingsForm)
#[tauri::command]
pub fn get_sync_config(
    app_handle: tauri::AppHandle,
) -> Result<Option<SyncConfig>, String> {
    get_cloud_sync_config(app_handle)
}

#[derive(serde::Serialize)]
pub struct AppSyncStatus {
    last_sync_time: Option<String>,
    sync_count: i32,
}

/// Get sync status (latest sync time across all tables)
#[tauri::command]
pub async fn get_sync_status(
    state: tauri::State<'_, crate::AppState>,
) -> Result<AppSyncStatus, String> {
    use rusqlite::OptionalExtension;
    
    let conn_guard = state.db_state.conn.lock().await;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    
    // Get the latest sync time from any table
    let last_sync: Option<String> = conn.query_row(
        "SELECT MAX(last_sync_time) FROM sync_status",
        [],
        |row| row.get(0)
    ).optional().map_err(|e| e.to_string())?.flatten();
    
    // Get total successful syncs (sum of counts)
    let total_count: i32 = conn.query_row(
        "SELECT COALESCE(SUM(sync_count), 0) FROM sync_status",
        [],
        |row| row.get(0)
    ).unwrap_or(0);
    
    Ok(AppSyncStatus {
        last_sync_time: last_sync,
        sync_count: total_count,
    })
}
