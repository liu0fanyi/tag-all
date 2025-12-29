//! Workspace Commands
//!
//! Tauri commands for workspace management.

use tauri::State;

use crate::AppState;
use crate::domain::Workspace;

#[tauri::command]
pub async fn list_workspaces(
    state: State<'_, AppState>,
) -> Result<Vec<Workspace>, String> {
    let repo = state.workspace_repo.lock().await;
    repo.list().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_workspace(
    name: String,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    let repo = state.workspace_repo.lock().await;
    repo.create(&name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_workspace(
    id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let repo = state.workspace_repo.lock().await;
    repo.delete(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_workspace(
    id: u32,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let repo = state.workspace_repo.lock().await;
    repo.rename(id, &name).await.map_err(|e| e.to_string())
}
