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

mod domain;
mod repository;
mod commands;

use repository::{ItemRepository, init_db};

/// Application state shared across commands
pub struct AppState {
    pub item_repo: Mutex<ItemRepository>,
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
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Initialize database
            tauri::async_runtime::block_on(async move {
                let db_path = get_db_path(&app_handle);
                let db_state = init_db(&db_path).await.expect("Failed to init database");
                
                // Create connection for repository
                let conn = db_state.get_connection().await.expect("Failed to get connection");
                let item_repo = ItemRepository::new(Arc::new(Mutex::new(conn)));
                
                // Store state
                app_handle.manage(AppState {
                    item_repo: Mutex::new(item_repo),
                });
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_item,
            commands::list_items,
            commands::get_item,
            commands::update_item,
            commands::delete_item,
            commands::toggle_item,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
