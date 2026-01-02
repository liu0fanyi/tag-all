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

