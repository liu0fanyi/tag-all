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
