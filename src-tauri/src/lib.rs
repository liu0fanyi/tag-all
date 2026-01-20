//! Tag-All Backend
//!
//! Layered architecture:
//! - domain: Core entities and business rules
//! - repository: Data access abstractions and implementations
//! - commands: Tauri command handlers

use std::path::PathBuf;
use tauri::{Manager, Emitter};
use percent_encoding::percent_decode_str;

mod domain;
mod repository;
mod commands;

use repository::{init_db, DbState};

/// Application state shared across commands
pub struct AppState {
    pub db_state: DbState,
    pub db_path: PathBuf,
}

/// Get database path from app handle
fn get_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let app_dir = app_handle.path().app_data_dir().unwrap();
    std::fs::create_dir_all(&app_dir).unwrap();
    app_dir.join("tag_all.db")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .register_asynchronous_uri_scheme_protocol("asset", |_ctx, request, responder| {
            std::thread::spawn(move || {
                let path = request.uri().path();
                // Decode path (percent-decoded)
                let path = percent_decode_str(path)
                    .decode_utf8_lossy()
                    .to_string();

                // Handle Windows paths: /C:/Users... -> C:/Users...
                let path = if path.starts_with('/') && path.chars().nth(2) == Some(':') {
                    path[1..].to_string()
                } else {
                    path
                };

                let path = std::path::PathBuf::from(&path);
                
                if !path.exists() {
                    let response = tauri::http::Response::builder()
                        .status(404)
                        .body(Vec::new())
                        .expect("Failed to build 404 response");
                    responder.respond(response);
                    return;
                }
                
                match std::fs::read(&path) {
                    Ok(content) => {
                        let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
                        let response = tauri::http::Response::builder()
                            .header("Content-Type", mime_type.as_ref())
                            .header("Access-Control-Allow-Origin", "*")
                            .body(content)
                            .expect("Failed to build response");
                        responder.respond(response);
                    }
                    Err(_) => {
                        let response = tauri::http::Response::builder()
                            .status(500)
                            .body(Vec::new())
                            .expect("Failed to build 500 response");
                        responder.respond(response);
                    }
                }
            });
        })
        .setup(|app| {
            // Single instance check - must be first!
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
                // Focus the existing window when a new instance tries to start
                #[cfg(desktop)]
                if let Some(window) = _app.get_webview_window("main") {
                    let _ = window.set_focus();
                }
            }))?;

            let app_handle = app.handle().clone();

            // Initialize logging
            rolling_logger::init_logger(
                app_handle.path().app_log_dir().expect("failed to get log dir"),
                "TagAll"
            ).expect("failed to init rolling logger");
            
            let db_path = get_db_path(&app_handle);
            
            eprintln!("[{}] App setup starting", chrono::Local::now().format("%H:%M:%S%.3f"));
            
            // Create initial empty DbState (managed)
            let db_state = DbState::new(db_path.clone());
            
            // Manage state IMMEDIATELY
            app.manage(AppState {
                db_state: db_state.clone(),
                db_path: db_path.clone(),
            });
            
            eprintln!("[{}] State managed, app will start immediately", chrono::Local::now().format("%H:%M:%S%.3f"));
            
            // Initialize database asynchronously in background
            tauri::async_runtime::spawn(async move {
                eprintln!("[{}] Background: Starting DB initialization", chrono::Local::now().format("%H:%M:%S%.3f"));
                
                match init_db(&db_path).await {
                    Ok(initialized_state) => {
                        eprintln!("[{}] Background: DB initialized successfully", chrono::Local::now().format("%H:%M:%S%.3f"));
                        let _ = rolling_logger::info("Async DB init success");
                        
                        // Update the existing DbState with the initialized data
                        {
                            // 1. Connection
                            let mut conn_guard = db_state.conn.lock().await;
                            *conn_guard = initialized_state.conn.lock().await.take();
                            
                            // 2. Sync Config - skipped if not available
                        }
                        
                        eprintln!("[{}] Background: DbState updated", chrono::Local::now().format("%H:%M:%S%.3f"));
                        
                        // Emit event to notify frontend
                        eprintln!("[{}] Background: Emitting db-initialized event", chrono::Local::now().format("%H:%M:%S%.3f"));
                        if let Err(e) = app_handle.emit("db-initialized", ()) {
                            eprintln!("Failed to emit event: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("[{}] Background: DB init failed: {}", chrono::Local::now().format("%H:%M:%S%.3f"), e);
                        let _ = rolling_logger::error(&format!("Async DB init failed: {}", e));
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Level 1-2: Item CRUD + Hierarchy
            commands::create_item,
            commands::list_items,
            commands::list_items_by_workspace,
            commands::get_item,
            commands::update_item,
            commands::delete_item,
            commands::toggle_item,
            commands::get_children,
            commands::move_item,
            commands::toggle_collapsed,
            commands::get_descendants,
            commands::decrement_item,
            commands::set_item_count,
            commands::reset_all_items,
            // Level 3: Tag CRUD + Item-Tag relationships
            commands::create_tag,
            commands::list_tags,
            commands::get_tag,
            commands::update_tag,
            commands::delete_tag,
            commands::add_item_tag,
            commands::remove_item_tag,
            commands::get_item_tags,
            commands::get_items_by_tag,
            // Level 3: Tag-Tag relationships (multi-parent)
            commands::add_tag_parent,
            commands::remove_tag_parent,
            commands::get_tag_parents,
            commands::get_tag_children,
            commands::get_root_tags,
            commands::move_tag,
            commands::move_child_tag,
            // Level 4: Window state
            commands::save_window_state,
            commands::load_window_state,
            commands::resize_window,
            commands::shrink_window,
            commands::set_pinned,
            commands::minimize_window,
            commands::close_window,
            // Level 5: Workspaces
            commands::list_workspaces,
            commands::create_workspace,
            commands::delete_workspace,
            commands::rename_workspace,
            commands::list_workspace_paths,
            commands::add_workspace_path,
            commands::remove_workspace_path,
            commands::toggle_workspace_dir_collapsed,
            // Cloud Sync
            commands::configure_cloud_sync,
            commands::get_cloud_sync_config,
            commands::save_cloud_sync_config,
            commands::configure_sync,
            commands::sync_cloud_db,
            commands::sync_database,
            commands::get_sync_config,
            commands::get_sync_status,
            commands::is_cloud_sync_enabled,
            // Level 7: Files
            commands::list_directory,
            commands::ensure_file_item,
            commands::pick_folder,
            commands::open_file,
            // Clipboard
            commands::save_clipboard_image,
            commands::clean_unused_assets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
