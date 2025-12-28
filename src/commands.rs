//! Tauri Command Wrappers
//!
//! Frontend bindings to backend commands.

use wasm_bindgen::prelude::*;
use serde::Serialize;
use crate::models::Item;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

// ========================
// Command Argument Structs
// ========================

#[derive(Serialize)]
pub struct CreateItemArgs<'a> {
    pub text: &'a str,
    #[serde(rename = "itemType")]
    pub item_type: Option<&'a str>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<u32>,
}

#[derive(Serialize)]
pub struct IdArgs {
    pub id: u32,
}

// ========================
// Command Functions
// ========================

pub async fn list_items() -> Result<Vec<Item>, String> {
    let result = invoke("list_items", JsValue::NULL).await;
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
