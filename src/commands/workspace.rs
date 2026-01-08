//! Workspace Commands
//!
//! Frontend bindings for workspace-related backend commands.

use wasm_bindgen::prelude::*;
use serde::Serialize;
use crate::models::Workspace;
use super::invoke;

// ========================
// Argument Structs
// ========================

#[derive(Serialize)]
struct CreateWorkspaceArgs<'a> {
    name: &'a str,
}

#[derive(Serialize)]
struct RenameWorkspaceArgs<'a> {
    id: u32,
    name: &'a str,
}

#[derive(Serialize)]
struct DeleteWorkspaceArgs {
    id: u32,
}

// ========================
// Commands
// ========================

pub async fn list_workspaces() -> Result<Vec<Workspace>, String> {
    let result = invoke("list_workspaces", JsValue::NULL).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn create_workspace(name: &str) -> Result<Workspace, String> {
    let js_args = serde_wasm_bindgen::to_value(&CreateWorkspaceArgs { name }).map_err(|e| e.to_string())?;
    let result = invoke("create_workspace", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn rename_workspace(id: u32, name: &str) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&RenameWorkspaceArgs { id, name }).map_err(|e| e.to_string())?;
    let result = invoke("rename_workspace", js_args).await;
    // Result is () on success
    if result.is_undefined() || result.is_null() {
        Ok(())
    } else {
        serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
    }
}

pub async fn delete_workspace(id: u32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&DeleteWorkspaceArgs { id }).map_err(|e| e.to_string())?;
    let result = invoke("delete_workspace", js_args).await;
    // Result is () on success
    if result.is_undefined() || result.is_null() {
        Ok(())
    } else {
        serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
    }
}

// Workspace Dir Commands

#[derive(Serialize)]
struct ListWorkspacePathsArgs {
    #[serde(rename = "workspaceId")]
    workspace_id: u32,
}

#[derive(Serialize)]
struct AddWorkspacePathArgs<'a> {
    #[serde(rename = "workspaceId")]
    workspace_id: u32,
    path: &'a str,
}

#[derive(Serialize)]
struct RemoveWorkspacePathArgs {
    id: u32,
}

pub async fn list_workspace_paths(workspace_id: u32) -> Result<Vec<crate::models::WorkspaceDir>, String> {
    let js_args = serde_wasm_bindgen::to_value(&ListWorkspacePathsArgs { workspace_id }).map_err(|e| e.to_string())?;
    let result = invoke("list_workspace_paths", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn add_workspace_path(workspace_id: u32, path: &str) -> Result<crate::models::WorkspaceDir, String> {
    // Note: Parameter name in backend argument might be camelCase on frontend side if not configured strictly, 
    // but Tauri usually expects exact match or camelCase conversion. 
    // Usually Rust struct fields are snake_case, Tauri converts to camelCase for JS, but here we invoke with JS object.
    // serde_wasm_bindgen converts struct fields to JS properties. 
    // Backend expects arguments by name.
    // Backend arg names: workspace_id, path.
    // Struct ListWorkspacePathsArgs { workspace_id } -> JS { workspaceId: ... }
    // Tauri auto-converts camelCase to snake_case for args. So workspaceId works for workspace_id.
    
    let js_args = serde_wasm_bindgen::to_value(&AddWorkspacePathArgs { workspace_id, path }).map_err(|e| e.to_string())?;
    let result = invoke("add_workspace_path", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn remove_workspace_path(id: u32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&RemoveWorkspacePathArgs { id }).map_err(|e| e.to_string())?;
    let result = invoke("remove_workspace_path", js_args).await;
    if result.is_undefined() || result.is_null() {
        Ok(())
    } else {
        serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
    }
}

#[derive(Serialize)]
struct ToggleWorkspaceDirCollapsedArgs {
    id: u32,
    collapsed: bool,
}

pub async fn toggle_workspace_dir_collapsed(id: u32, collapsed: bool) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&ToggleWorkspaceDirCollapsedArgs { id, collapsed }).map_err(|e| e.to_string())?;
    let result = invoke("toggle_workspace_dir_collapsed", js_args).await;
    if result.is_undefined() || result.is_null() {
        Ok(())
    } else {
        serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
    }
}
