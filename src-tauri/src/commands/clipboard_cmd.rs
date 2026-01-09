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
