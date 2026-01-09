//! Tag-All Backend
//!
//! Layered architecture:
//! - domain: Core entities and business rules
//! - repository: Data access abstractions and implementations
//! - commands: Tauri command handlers

use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use percent_encoding::percent_decode_str;

mod domain;
mod repository;
mod commands;

use repository::{ItemRepository, TagRepository, WindowStateRepository, WorkspaceRepository, init_db, DbState};

/// Application state shared across commands
pub struct AppState {
    pub item_repo: Mutex<ItemRepository>,
    pub tag_repo: Mutex<TagRepository>,
    pub window_repo: Mutex<WindowStateRepository>,
    pub workspace_repo: Mutex<WorkspaceRepository>,
    pub db_state: DbState,
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
            let app_handle = app.handle().clone();
            
            // Initialize database
            tauri::async_runtime::block_on(async move {
                let db_path = get_db_path(&app_handle);
                let db_state = init_db(&db_path).await.expect("Failed to init database");
                
                // Create connection for repositories
                let conn = db_state.get_connection().await.expect("Failed to get connection");
                let conn = Arc::new(Mutex::new(conn));
                
                let item_repo = ItemRepository::new(conn.clone());
                let tag_repo = TagRepository::new(conn.clone());
                let window_repo = WindowStateRepository::new(conn.clone());
                let workspace_repo = WorkspaceRepository::new(conn.clone());
                
                // Store state
                app_handle.manage(AppState {
                    item_repo: Mutex::new(item_repo),
                    tag_repo: Mutex::new(tag_repo),
                    window_repo: Mutex::new(window_repo),
                    workspace_repo: Mutex::new(workspace_repo),
                    db_state,
                });
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
            commands::sync_cloud_db,
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
