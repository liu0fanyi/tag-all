//! Tauri Command Wrappers
//!
//! Frontend bindings to backend commands.

use wasm_bindgen::prelude::*;
use serde::Serialize;
use crate::models::{Item, Tag};

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

#[derive(Serialize)]
pub struct ItemIdArgs {
    #[serde(rename = "itemId")]
    pub item_id: u32,
}

#[derive(Serialize)]
pub struct TagIdArgs {
    #[serde(rename = "tagId")]
    pub tag_id: u32,
}

#[derive(Serialize)]
pub struct ParentTagIdArgs {
    #[serde(rename = "parentTagId")]
    pub parent_tag_id: u32,
}

#[derive(Serialize)]
pub struct CreateTagArgs<'a> {
    pub name: &'a str,
    pub color: Option<&'a str>,
}

#[derive(Serialize)]
pub struct ItemTagArgs {
    #[serde(rename = "itemId")]
    pub item_id: u32,
    #[serde(rename = "tagId")]
    pub tag_id: u32,
}

// ========================
// Item Commands
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

// ========================
// Tag Commands
// ========================

pub async fn list_tags() -> Result<Vec<Tag>, String> {
    let result = invoke("list_tags", JsValue::NULL).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn create_tag(args: &CreateTagArgs<'_>) -> Result<Tag, String> {
    let js_args = serde_wasm_bindgen::to_value(args).map_err(|e| e.to_string())?;
    let result = invoke("create_tag", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn delete_tag(id: u32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&IdArgs { id }).map_err(|e| e.to_string())?;
    let _ = invoke("delete_tag", js_args).await;
    Ok(())
}

pub async fn get_item_tags(item_id: u32) -> Result<Vec<Tag>, String> {
    let js_args = serde_wasm_bindgen::to_value(&ItemIdArgs { item_id }).map_err(|e| e.to_string())?;
    let result = invoke("get_item_tags", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn add_item_tag(item_id: u32, tag_id: u32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&ItemTagArgs { item_id, tag_id }).map_err(|e| e.to_string())?;
    let _ = invoke("add_item_tag", js_args).await;
    Ok(())
}

pub async fn remove_item_tag(item_id: u32, tag_id: u32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&ItemTagArgs { item_id, tag_id }).map_err(|e| e.to_string())?;
    let _ = invoke("remove_item_tag", js_args).await;
    Ok(())
}

// ========================
// Tag-Tag Commands (multi-parent)
// ========================

#[derive(Serialize)]
pub struct TagTagArgs {
    #[serde(rename = "childTagId")]
    pub child_tag_id: u32,
    #[serde(rename = "parentTagId")]
    pub parent_tag_id: u32,
}

pub async fn get_root_tags() -> Result<Vec<Tag>, String> {
    let result = invoke("get_root_tags", JsValue::NULL).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn get_tag_children(parent_tag_id: u32) -> Result<Vec<Tag>, String> {
    let js_args = serde_wasm_bindgen::to_value(&ParentTagIdArgs { parent_tag_id }).map_err(|e| e.to_string())?;
    let result = invoke("get_tag_children", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn get_tag_parents(tag_id: u32) -> Result<Vec<Tag>, String> {
    let js_args = serde_wasm_bindgen::to_value(&TagIdArgs { tag_id }).map_err(|e| e.to_string())?;
    let result = invoke("get_tag_parents", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn add_tag_parent(child_tag_id: u32, parent_tag_id: u32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&TagTagArgs { child_tag_id, parent_tag_id }).map_err(|e| e.to_string())?;
    let _ = invoke("add_tag_parent", js_args).await;
    Ok(())
}

pub async fn remove_tag_parent(child_tag_id: u32, parent_tag_id: u32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&TagTagArgs { child_tag_id, parent_tag_id }).map_err(|e| e.to_string())?;
    let _ = invoke("remove_tag_parent", js_args).await;
    Ok(())
}

// ========================
// Level 4: DnD + Window State Commands
// ========================

#[derive(Serialize)]
pub struct MoveItemArgs {
    pub id: u32,
    #[serde(rename = "newParentId")]
    pub new_parent_id: Option<u32>,
    pub position: i32,
}

pub async fn move_item(id: u32, new_parent_id: Option<u32>, position: i32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&MoveItemArgs { id, new_parent_id, position }).map_err(|e| e.to_string())?;
    let _ = invoke("move_item", js_args).await;
    Ok(())
}

#[derive(Serialize)]
pub struct MoveTagArgs {
    pub id: u32,
    pub position: i32,
}

pub async fn move_tag(id: u32, position: i32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&MoveTagArgs { id, position }).map_err(|e| e.to_string())?;
    let _ = invoke("move_tag", js_args).await;
    Ok(())
}

#[derive(Serialize)]
pub struct MoveChildTagArgs {
    #[serde(rename = "childTagId")]
    pub child_tag_id: u32,
    #[serde(rename = "parentTagId")]
    pub parent_tag_id: u32,
    pub position: i32,
}

pub async fn move_child_tag(child_tag_id: u32, parent_tag_id: u32, position: i32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&MoveChildTagArgs { child_tag_id, parent_tag_id, position }).map_err(|e| e.to_string())?;
    let _ = invoke("move_child_tag", js_args).await;
    Ok(())
}

#[derive(Serialize)]
pub struct WindowStateArgs {
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
    pub pinned: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WindowState {
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
    pub pinned: bool,
}

pub async fn save_window_state(width: f64, height: f64, x: f64, y: f64, pinned: bool) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&WindowStateArgs { width, height, x, y, pinned }).map_err(|e| e.to_string())?;
    let _ = invoke("save_window_state", js_args).await;
    Ok(())
}

pub async fn load_window_state() -> Result<Option<WindowState>, String> {
    let result = invoke("load_window_state", JsValue::NULL).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}
