//! File management commands
//!
//! Handles listing files, scanning directories, and resolving file identity.

use crate::repository::{TagRepository, ItemRepository};
use tauri::State;
use std::path::{Path, PathBuf};
use std::fs;
use serde::Serialize;
use crate::AppState;
use crate::domain::{Item, Tag, FileIdentifier};
use crate::repository::Repository;
use crate::repository::item::ItemWorkspaceOperations;
use crate::repository::tag::ItemTagOperations; // Import ItemTagOperations

#[derive(Debug, Serialize)]
pub struct FileViewItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub last_modified: u64,
    pub quick_hash: String,
    /// The database item if one matches this file
    pub db_item: Option<Item>,
    /// Associated tags
    pub tags: Vec<Tag>,
}

#[tauri::command]
pub async fn list_directory(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<FileViewItem>, String> {
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }

    let item_repo = ItemRepository::new(state.db_state.conn.clone());
    let tag_repo = TagRepository::new(state.db_state.conn.clone());

    // Use read_dir to scan directory
    let entries = fs::read_dir(dir_path).map_err(|e| e.to_string())?;
    
    let mut results = Vec::new();
    
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let is_dir = metadata.is_dir();
        let size = metadata.len();
        let last_modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
            
        let path_str = path.to_string_lossy().replace("\\", "/"); // Normalize slashes
        
        // Calculate quick hash
        let quick_hash = if is_dir {
            FileIdentifier::compute_quick_hash(&path).unwrap_or_default()
        } else {
            FileIdentifier::compute_quick_hash(&path).unwrap_or_default()
        };
        
        // Resolve Identity
        // 1. Try by Path
        let mut db_item = item_repo.find_by_last_known_path(&path_str).await.map_err(|e| e.to_string())?;
        
        if db_item.is_none() {
             // 2. Try by Quick Hash
             if !quick_hash.is_empty() {
                db_item = item_repo.find_by_quick_hash(&quick_hash, is_dir).await.map_err(|e| e.to_string())?;
             }
        }
        
        // Fetch tags if item exists
        let tags = if let Some(item) = &db_item {
            tag_repo.get_tags_for_item(item.id).await.unwrap_or_default()
        } else {
            Vec::new()
        };
        
        results.push(FileViewItem {
            name,
            path: path_str,
            is_dir,
            size,
            last_modified,
            quick_hash,
            db_item,
            tags,
        });
    }
    
    // Sort directories first, then files
    results.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.cmp(&b.name)
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(results)
}

/// Start tagging a file (Calculates strict content hash and ensures Item exists)
#[tauri::command]
pub async fn ensure_file_item(
    state: State<'_, AppState>,
    path: String,
) -> Result<Item, String> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err("File does not exist".to_string());
    }

    let item_repo = ItemRepository::new(state.db_state.conn.clone());

    // 1. Calculate Content Hash (definitive identity)
    // For directories, we can't read content, so we use quick_hash as the content hash
    let is_dir = path_buf.is_dir();
    let content_hash = if is_dir {
        FileIdentifier::compute_quick_hash(&path_buf)?
    } else {
        FileIdentifier::compute_content_hash(&path_buf)?
    };
    
    // 2. Check DB by Content Hash
    if let Some(mut item) = item_repo.find_by_content_hash(&content_hash).await.map_err(|e| e.to_string())? {
        // Update Metadata if changed
        let quick_hash = FileIdentifier::compute_quick_hash(&path_buf).unwrap_or_default();
        let path_str = path.replace("\\", "/");
        
        if item.last_known_path.as_deref() != Some(&path_str) || item.quick_hash.as_deref() != Some(&quick_hash) {
            item.last_known_path = Some(path_str);
            item.quick_hash = Some(quick_hash);
            
            item_repo.update(&item).await.map_err(|e| e.to_string())?;
            return Ok(item);
        } else {
             return Ok(item);
        }
    }
    
    // 3. Check DB by Path (Collision or Content Changed)
    let path_str = path.replace("\\", "/");
    
    if let Some(mut item) = item_repo.find_by_last_known_path(&path_str).await.map_err(|e| e.to_string())? {
        // Update content hash and quick hash received from new computation
        let quick_hash = FileIdentifier::compute_quick_hash(&path_buf).unwrap_or_default();
        
        item.content_hash = Some(content_hash);
        item.quick_hash = Some(quick_hash);
        
        item_repo.update(&item).await.map_err(|e| e.to_string())?;
        return Ok(item);
    }

    // 4. Create New Item
    let file_name = path_buf.file_name().unwrap_or_default().to_string_lossy().to_string();
    let quick_hash = FileIdentifier::compute_quick_hash(&path_buf).unwrap_or_default();
    let is_dir = path_buf.is_dir();

    let mut item = Item::new(0, file_name, crate::domain::ItemType::Document); // Default to Document type for files
    item.content_hash = Some(content_hash);
    item.quick_hash = Some(quick_hash);
    item.last_known_path = Some(path_str);
    item.is_dir = is_dir;
    
    // Assign to 'files' workspace (ID 2)
    item_repo.create_with_workspace(&item, 2).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    open::that(path).map_err(|e| e.to_string())
}
