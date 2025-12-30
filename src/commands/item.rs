//! Item Commands
//!
//! Frontend bindings for item-related backend commands.

use wasm_bindgen::prelude::*;
use serde::Serialize;
use crate::models::Item;
use super::invoke;

// ========================
// Argument Structs
// ========================

#[derive(Serialize)]
pub struct CreateItemArgs<'a> {
    pub text: &'a str,
    #[serde(rename = "itemType")]
    pub item_type: Option<&'a str>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<u32>,
    #[serde(rename = "workspaceId")]
    pub workspace_id: Option<u32>,
}

#[derive(Serialize)]
struct IdArgs {
    id: u32,
}

#[derive(Serialize)]
struct MoveItemArgs {
    id: u32,
    #[serde(rename = "newParentId")]
    new_parent_id: Option<u32>,
    position: i32,
}

#[derive(Serialize)]
struct WorkspaceIdArgs {
    #[serde(rename = "workspaceId")]
    workspace_id: u32,
}

// ========================
// Commands
// ========================

pub async fn list_items() -> Result<Vec<Item>, String> {
    let result = invoke("list_items", JsValue::NULL).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn list_items_by_workspace(workspace_id: u32) -> Result<Vec<Item>, String> {
    let js_args = serde_wasm_bindgen::to_value(&WorkspaceIdArgs { workspace_id }).map_err(|e| e.to_string())?;
    let result = invoke("list_items_by_workspace", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn create_item(args: &CreateItemArgs<'_>) -> Result<Item, String> {
    let js_args = serde_wasm_bindgen::to_value(args).map_err(|e| e.to_string())?;
    let result = invoke("create_item", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn toggle_item(id: u32) -> Result<Item, String> {
    let js_args = serde_wasm_bindgen::to_value(&IdArgs { id }).map_err(|e| e.to_string())?;
    let result = invoke("toggle_item", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn delete_item(id: u32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&IdArgs { id }).map_err(|e| e.to_string())?;
    let _ = invoke("delete_item", js_args).await;
    Ok(())
}

pub async fn toggle_collapsed(id: u32) -> Result<bool, String> {
    let js_args = serde_wasm_bindgen::to_value(&IdArgs { id }).map_err(|e| e.to_string())?;
    let result = invoke("toggle_collapsed", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn move_item(id: u32, new_parent_id: Option<u32>, position: i32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&MoveItemArgs { id, new_parent_id, position }).map_err(|e| e.to_string())?;
    let _ = invoke("move_item", js_args).await;
    Ok(())
}

pub async fn get_item(id: u32) -> Result<Option<Item>, String> {
    let js_args = serde_wasm_bindgen::to_value(&IdArgs { id }).map_err(|e| e.to_string())?;
    let result = invoke("get_item", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct UpdateItemArgs<'a> {
    id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    completed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    item_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memo: Option<&'a str>,
}

pub async fn update_item(id: u32, text: Option<&str>, item_type: Option<&str>) -> Result<Item, String> {
    // Build JSON string with camelCase for Tauri IPC
    let mut json = format!(r#"{{"id":{}"#, id);
    if let Some(t) = text {
        json.push_str(&format!(r#","text":"{}""#, t));
    }
    if let Some(it) = item_type {
        json.push_str(&format!(r#","itemType":"{}""#, it));  // camelCase for Tauri
    }
    json.push('}');
    
    let js_args = js_sys::JSON::parse(&json).map_err(|e| format!("JSON parse error: {:?}", e))?;
    let result = invoke("update_item", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn decrement_item(id: u32) -> Result<Item, String> {
    let js_args = serde_wasm_bindgen::to_value(&IdArgs { id }).map_err(|e| e.to_string())?;
    let result = invoke("decrement_item", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct SetItemCountArgs {
    id: u32,
    target_count: Option<i32>,
}

pub async fn set_item_count(id: u32, target_count: Option<i32>) -> Result<Item, String> {
    // Build JSON string with camelCase for Tauri IPC
    let mut json = format!(r#"{{"id":{}"#, id);
    
    if let Some(count) = target_count {
        json.push_str(&format!(r#","targetCount":{}"#, count));  // camelCase!
    }
    json.push('}');
    
    let js_args = js_sys::JSON::parse(&json).map_err(|e| format!("JSON parse error: {:?}", e))?;
    let result = invoke("set_item_count", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

/// Update item memo (Markdown content)
pub async fn update_item_memo(id: u32, memo: Option<&str>) -> Result<Item, String> {
    // Build JSON with memo field - escape the memo content for JSON
    let mut json = format!(r#"{{"id":{}"#, id);
    
    if let Some(m) = memo {
        // Escape special JSON characters in memo
        let escaped = m
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        json.push_str(&format!(r#","memo":"{}""#, escaped));
    } else {
        json.push_str(r#","memo":null"#);
    }
    json.push('}');
    
    let js_args = js_sys::JSON::parse(&json).map_err(|e| format!("JSON parse error: {:?}", e))?;
    let result = invoke("update_item", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

/// Reset all completed items in a workspace back to incomplete
pub async fn reset_all_items(workspace_id: u32) -> Result<u32, String> {
    let js_args = serde_wasm_bindgen::to_value(&WorkspaceIdArgs { workspace_id }).map_err(|e| e.to_string())?;
    let result = invoke("reset_all_items", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}
