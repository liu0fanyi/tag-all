//! Window State Commands
//!
//! Tauri commands for saving/loading window state and resizing.

use tauri::{State, AppHandle, Manager};
use crate::AppState;
use crate::repository::{WindowState, WindowStateRepository};

/// Save window state
#[tauri::command]
pub async fn save_window_state(
    state: State<'_, AppState>,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
    pinned: bool,
) -> Result<(), String> {
    let repo = WindowStateRepository::new(state.db_state.conn.clone());
    
    let window_state = WindowState {
        width,
        height,
        x,
        y,
        pinned,
    };
    
    repo.save(&window_state).await
}

/// Load window state
#[tauri::command]
pub async fn load_window_state(state: State<'_, AppState>) -> Result<Option<WindowState>, String> {
    let repo = WindowStateRepository::new(state.db_state.conn.clone());
    repo.load().await
}

/// Resize main window to specified size (only expands, doesn't shrink)
/// Skips resizing if window is currently maximized to prevent accidental restore
#[tauri::command]
pub async fn resize_window(app: AppHandle, width: u32, height: u32) -> Result<(), String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let window = app.get_webview_window("main").ok_or("Window not found")?;
        
        // Don't resize if maximized (would cause unwanted restore)
        if window.is_maximized().unwrap_or(false) {
            return Ok(());
        }
        
        // Get current size
        let current_size = window.outer_size().map_err(|e| e.to_string())?;
        
        // Only resize if new size is larger (expand, don't shrink)
        let new_width = width.max(current_size.width);
        let new_height = height.max(current_size.height);
        
        if new_width != current_size.width || new_height != current_size.height {
            window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: new_width,
                height: new_height,
            })).map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

/// Shrink window to specified size (allows reducing size)
/// Skips resizing if window is currently maximized to prevent accidental restore
#[tauri::command]
pub async fn shrink_window(app: AppHandle, width: u32, height: u32) -> Result<(), String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let window = app.get_webview_window("main").ok_or("Window not found")?;
        
        // Don't shrink if maximized (would cause unwanted restore)
        if window.is_maximized().unwrap_or(false) {
            return Ok(());
        }

        // Get current size to preserve height if it's taller
        let current_size = window.outer_size().map_err(|e| e.to_string())?;

        // Use the larger of current height or target height
        let new_height = height.max(current_size.height);
        
        window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width,
            height: new_height,
        })).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// Set window always-on-top state
#[tauri::command]
pub async fn set_pinned(app: AppHandle, pinned: bool) -> Result<(), String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let window = app.get_webview_window("main").ok_or("Window not found")?;
        window.set_always_on_top(pinned).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Minimize window
#[tauri::command]
pub async fn minimize_window(app: AppHandle) -> Result<(), String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let window = app.get_webview_window("main").ok_or("Window not found")?;
        window.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Close window
#[tauri::command]
pub async fn close_window(app: AppHandle) -> Result<(), String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let window = app.get_webview_window("main").ok_or("Window not found")?;
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
