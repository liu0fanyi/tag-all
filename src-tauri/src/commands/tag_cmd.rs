//! Tauri Commands for Tag operations
//!
//! Exposes Tag CRUD and item-tag relationships to the frontend.

use tauri::State;
use crate::domain::Tag;
use crate::repository::{Repository, TagRepository};
use crate::repository::tag::{ItemTagOperations, TagHierarchyOperations, TagPositioningOperations};
use crate::AppState;

/// Create a new tag
#[tauri::command]
pub async fn create_tag(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
) -> Result<Tag, String> {
    let repo = state.tag_repo.lock().await;
    
    let tag = if let Some(c) = color {
        Tag::with_color(0, name, c)
    } else {
        Tag::new(0, name)
    };
    
    repo.create(&tag).await.map_err(|e| e.to_string())
}

/// List all tags
#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let repo = state.tag_repo.lock().await;
    repo.list().await.map_err(|e| e.to_string())
}

/// Get tag by ID
#[tauri::command]
pub async fn get_tag(state: State<'_, AppState>, id: u32) -> Result<Option<Tag>, String> {
    let repo = state.tag_repo.lock().await;
    repo.find_by_id(id).await.map_err(|e| e.to_string())
}

/// Update tag
#[tauri::command]
pub async fn update_tag(
    state: State<'_, AppState>,
    id: u32,
    name: Option<String>,
    color: Option<String>,
) -> Result<Tag, String> {
    let repo = state.tag_repo.lock().await;
    
    let existing = repo.find_by_id(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Tag {} not found", id))?;
    
    let updated = Tag {
        id: existing.id,
        name: name.unwrap_or(existing.name),
        color: color.or(existing.color),
        position: existing.position,
    };
    
    repo.update(&updated).await.map_err(|e| e.to_string())
}

/// Delete tag
#[tauri::command]
pub async fn delete_tag(state: State<'_, AppState>, id: u32) -> Result<(), String> {
    let repo = state.tag_repo.lock().await;
    repo.delete(id).await.map_err(|e| e.to_string())
}

// ========================
// Item-Tag Relationships
// ========================

/// Add a tag to an item
#[tauri::command]
pub async fn add_item_tag(
    state: State<'_, AppState>,
    item_id: u32,
    tag_id: u32,
) -> Result<(), String> {
    let repo = state.tag_repo.lock().await;
    repo.add_tag_to_item(item_id, tag_id).await.map_err(|e| e.to_string())
}

/// Remove a tag from an item
#[tauri::command]
pub async fn remove_item_tag(
    state: State<'_, AppState>,
    item_id: u32,
    tag_id: u32,
) -> Result<(), String> {
    let repo = state.tag_repo.lock().await;
    repo.remove_tag_from_item(item_id, tag_id).await.map_err(|e| e.to_string())
}

/// Get all tags for an item
#[tauri::command]
pub async fn get_item_tags(
    state: State<'_, AppState>,
    item_id: u32,
) -> Result<Vec<Tag>, String> {
    let repo = state.tag_repo.lock().await;
    repo.get_tags_for_item(item_id).await.map_err(|e| e.to_string())
}

/// Get all item IDs with a specific tag
#[tauri::command]
pub async fn get_items_by_tag(
    state: State<'_, AppState>,
    tag_id: u32,
) -> Result<Vec<u32>, String> {
    let repo = state.tag_repo.lock().await;
    repo.get_items_with_tag(tag_id).await.map_err(|e| e.to_string())
}

// ========================
// Tag-Tag Relationships (multi-parent)
// ========================

/// Add a parent tag to a child tag (tag the child with the parent)
#[tauri::command]
pub async fn add_tag_parent(
    state: State<'_, AppState>,
    child_tag_id: u32,
    parent_tag_id: u32,
) -> Result<(), String> {
    let repo = state.tag_repo.lock().await;
    repo.add_parent_tag(child_tag_id, parent_tag_id).await.map_err(|e| e.to_string())
}

/// Remove a parent tag from a child tag
#[tauri::command]
pub async fn remove_tag_parent(
    state: State<'_, AppState>,
    child_tag_id: u32,
    parent_tag_id: u32,
) -> Result<(), String> {
    let repo = state.tag_repo.lock().await;
    repo.remove_parent_tag(child_tag_id, parent_tag_id).await.map_err(|e| e.to_string())
}

/// Get all parent tags for a given tag
#[tauri::command]
pub async fn get_tag_parents(
    state: State<'_, AppState>,
    tag_id: u32,
) -> Result<Vec<Tag>, String> {
    let repo = state.tag_repo.lock().await;
    repo.get_parent_tags(tag_id).await.map_err(|e| e.to_string())
}

/// Get all child tags for a given parent tag
#[tauri::command]
pub async fn get_tag_children(
    state: State<'_, AppState>,
    parent_tag_id: u32,
) -> Result<Vec<Tag>, String> {
    let repo = state.tag_repo.lock().await;
    repo.get_child_tags(parent_tag_id).await.map_err(|e| e.to_string())
}

/// Get root tags (tags with no parents)
#[tauri::command]
pub async fn get_root_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let repo = state.tag_repo.lock().await;
    repo.get_root_tags().await.map_err(|e| e.to_string())
}

/// Move a root tag to a new position
#[tauri::command]
pub async fn move_tag(
    state: State<'_, AppState>,
    id: u32,
    position: i32,
) -> Result<(), String> {
    let repo = state.tag_repo.lock().await;
    repo.move_tag(id, position).await.map_err(|e| e.to_string())
}

/// Move a child tag to a new position under a parent
#[tauri::command]
pub async fn move_child_tag(
    state: State<'_, AppState>,
    child_tag_id: u32,
    parent_tag_id: u32,
    position: i32,
) -> Result<(), String> {
    let repo = state.tag_repo.lock().await;
    repo.move_child_tag(child_tag_id, parent_tag_id, position).await.map_err(|e| e.to_string())
}
