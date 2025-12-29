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

use repository::{ItemRepository, TagRepository, WindowStateRepository, init_db};

/// Application state shared across commands
pub struct AppState {
    pub item_repo: Mutex<ItemRepository>,
    pub tag_repo: Mutex<TagRepository>,
    pub window_repo: Mutex<WindowStateRepository>,
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
                
                // Create connection for repositories
                let conn = db_state.get_connection().await.expect("Failed to get connection");
                let conn = Arc::new(Mutex::new(conn));
                
                let item_repo = ItemRepository::new(conn.clone());
                let tag_repo = TagRepository::new(conn.clone());
                let window_repo = WindowStateRepository::new(conn.clone());
                
                // Store state
                app_handle.manage(AppState {
                    item_repo: Mutex::new(item_repo),
                    tag_repo: Mutex::new(tag_repo),
                    window_repo: Mutex::new(window_repo),
                });
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Level 1-2: Item CRUD + Hierarchy
            commands::create_item,
            commands::list_items,
            commands::get_item,
            commands::update_item,
            commands::delete_item,
            commands::toggle_item,
            commands::get_children,
            commands::move_item,
            commands::toggle_collapsed,
            commands::get_descendants,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
