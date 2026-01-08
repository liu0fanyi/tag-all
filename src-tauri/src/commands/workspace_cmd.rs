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

#[tauri::command]
pub async fn list_workspace_paths(
    workspace_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<crate::domain::WorkspaceDir>, String> {
    let repo = state.workspace_repo.lock().await;
    repo.list_paths(workspace_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_workspace_path(
    workspace_id: u32,
    path: String,
    state: State<'_, AppState>,
) -> Result<crate::domain::WorkspaceDir, String> {
    let repo = state.workspace_repo.lock().await;
    repo.add_path(workspace_id, &path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_workspace_path(
    id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let repo = state.workspace_repo.lock().await;
    repo.remove_path(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_workspace_dir_collapsed(
    id: u32,
    collapsed: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let repo = state.workspace_repo.lock().await;
    repo.set_path_collapsed(id, collapsed).await.map_err(|e| e.to_string())
}
