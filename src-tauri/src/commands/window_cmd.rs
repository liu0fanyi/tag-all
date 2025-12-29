//! Window State Commands
//!
//! Tauri commands for saving/loading window state.

use tauri::State;
use crate::AppState;
use crate::repository::WindowState;

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
    let repo = state.window_repo.lock().await;
    
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
    let repo = state.window_repo.lock().await;
    repo.load().await
}
