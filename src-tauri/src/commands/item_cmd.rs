//! Tauri Commands for Item CRUD
//!
//! Exposes Item operations to the frontend via Tauri IPC.

use tauri::State;
use crate::domain::{Item, ItemType};
use crate::repository::{Repository, ItemRepository};
use crate::AppState;

/// Create a new item
#[tauri::command]
pub async fn create_item(
    state: State<'_, AppState>,
    text: String,
    item_type: Option<String>,
) -> Result<Item, String> {
    let repo = state.item_repo.lock().await;
    
    let item = Item::new(
        0, // ID will be assigned by database
        text,
        item_type.map(|t| ItemType::from_str(&t)).unwrap_or_default(),
    );
    
    repo.create(&item).await.map_err(|e| e.to_string())
}

/// List all items
#[tauri::command]
pub async fn list_items(state: State<'_, AppState>) -> Result<Vec<Item>, String> {
    let repo = state.item_repo.lock().await;
    repo.list().await.map_err(|e| e.to_string())
}

/// Get item by ID
#[tauri::command]
pub async fn get_item(state: State<'_, AppState>, id: u32) -> Result<Option<Item>, String> {
    let repo = state.item_repo.lock().await;
    repo.find_by_id(id).await.map_err(|e| e.to_string())
}

/// Update item
#[tauri::command]
pub async fn update_item(
    state: State<'_, AppState>,
    id: u32,
    text: Option<String>,
    completed: Option<bool>,
    item_type: Option<String>,
    memo: Option<String>,
) -> Result<Item, String> {
    let repo = state.item_repo.lock().await;
    
    // First get existing item
    let existing = repo.find_by_id(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Item {} not found", id))?;
    
    // Update fields
    let updated = Item {
        id: existing.id,
        text: text.unwrap_or(existing.text),
        completed: completed.unwrap_or(existing.completed),
        item_type: item_type.map(|t| ItemType::from_str(&t)).unwrap_or(existing.item_type),
        memo: memo.or(existing.memo),
        target_count: existing.target_count,
        current_count: existing.current_count,
    };
    
    repo.update(&updated).await.map_err(|e| e.to_string())
}

/// Delete item
#[tauri::command]
pub async fn delete_item(state: State<'_, AppState>, id: u32) -> Result<(), String> {
    let repo = state.item_repo.lock().await;
    repo.delete(id).await.map_err(|e| e.to_string())
}

/// Toggle item completion status
#[tauri::command]
pub async fn toggle_item(state: State<'_, AppState>, id: u32) -> Result<Item, String> {
    let repo = state.item_repo.lock().await;
    
    let mut item = repo.find_by_id(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Item {} not found", id))?;
    
    item.completed = !item.completed;
    
    // If it's a "once" type and completed, delete it
    if item.completed && item.item_type == ItemType::Once {
        repo.delete(id).await.map_err(|e| e.to_string())?;
        return Ok(item);
    }
    
    repo.update(&item).await.map_err(|e| e.to_string())
}
