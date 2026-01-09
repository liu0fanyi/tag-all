//! Clipboard Commands
//!
//! Commands for handling clipboard operations like saving pasted images.

use tauri::Manager;

/// Save clipboard image data to app data directory
/// 
/// Takes base64-encoded image data and saves it as a PNG file.
/// Returns the full path to the saved file.
#[tauri::command]
pub async fn save_clipboard_image(
    app_handle: tauri::AppHandle,
    data: String,
) -> Result<String, String> {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Get app data directory (same location as database)
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    // Create images subdirectory
    let images_dir = app_dir.join("clipboard_images");
    fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;
    
    // Generate unique filename using timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Time error: {}", e))?
        .as_millis();
    
    let filename = format!("{}.png", timestamp);
    let file_path = images_dir.join(&filename);
    
    // Decode base64 data
    // The data may have a data URL prefix like "data:image/png;base64,"
    let base64_data = if data.contains(",") {
        data.split(",").nth(1).unwrap_or(&data)
    } else {
        &data
    };
    
    let image_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        base64_data
    ).map_err(|e| format!("Failed to decode base64: {}", e))?;
    
    // Write to file
    fs::write(&file_path, image_bytes)
        .map_err(|e| format!("Failed to write image file: {}", e))?;
    
    // Return the full path as string
    let path_str = file_path.to_string_lossy().to_string();
    Ok(path_str)
}

/// Clean up unused assets from clipboard_images directory
/// 
/// Scans all items in the database for asset references.
/// Deletes files in clipboard_images that are not referenced by any item.
#[tauri::command]
pub async fn clean_unused_assets(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<usize, String> {
    use std::fs;
    use std::collections::HashSet;
    use regex::Regex;
    use crate::repository::traits::Repository;

    // 1. Get all items from DB
    let item_repo = state.item_repo.lock().await;
    let items: Vec<crate::domain::Item> = item_repo.list().await
        .map_err(|e| format!("Failed to list items: {}", e))?;
    
    // 2. Collect all used asset filenames
    // Regex to match markdown image/link syntax: ![](.../filename.png) or [](.../filename.png)
    // We specifically look for our clipboard_images path pattern
    // Path pattern: .../clipboard_images/timestamp.png
    // We just need to extract the filename really.
    let mut used_filenames = HashSet::new();
    
    // Pattern matches: "clipboard_images/" followed by non-closing-paren chars, then .png/.jpg/etc
    // Actually simpler: just search for the filename if we know they are in clipboard_images
    // Our save function produces "{timestamp}.png".
    // Let's match any reference that contains "clipboard_images/..."
    let re = Regex::new(r"clipboard_images[/\\]([^)\s]+)").unwrap();
    
    for item in items {
        if let Some(memo) = &item.memo {
            for cap in re.captures_iter(memo) {
                if let Some(match_str) = cap.get(1) {
                    // Collect filename
                    used_filenames.insert(match_str.as_str().to_string());
                }
            }
        }
    }

    // 3. List actual files in clipboard_images
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Get app dir failed: {}", e))?;
    let images_dir = app_dir.join("clipboard_images");
    
    if !images_dir.exists() {
        return Ok(0);
    }
    
    let entries = fs::read_dir(&images_dir)
        .map_err(|e| format!("Read dir failed: {}", e))?;
        
    let mut deleted_count = 0;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(filename_os) = path.file_name() {
                if let Some(filename) = filename_os.to_str() {
                    // Check if used
                    if !used_filenames.contains(filename) {
                        // Delete
                        // println!("Deleting unused asset: {}", filename);
                        if let Err(e) = fs::remove_file(&path) {
                            eprintln!("Failed to delete {}: {}", filename, e);
                        } else {
                            deleted_count += 1;
                        }
                    }
                }
            }
        }
    }
    
    Ok(deleted_count)
}
