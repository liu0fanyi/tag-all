//! Tag Commands
//!
//! Frontend bindings for tag-related backend commands.

use wasm_bindgen::prelude::*;
use serde::Serialize;
use crate::models::Tag;
use super::invoke;

// ========================
// Argument Structs
// ========================

#[derive(Serialize)]
pub struct CreateTagArgs<'a> {
    pub name: &'a str,
    pub color: Option<&'a str>,
}

#[derive(Serialize)]
struct IdArgs {
    id: u32,
}

#[derive(Serialize)]
struct ItemIdArgs {
    #[serde(rename = "itemId")]
    item_id: u32,
}

#[derive(Serialize)]
struct TagIdArgs {
    #[serde(rename = "tagId")]
    tag_id: u32,
}

#[derive(Serialize)]
struct ParentTagIdArgs {
    #[serde(rename = "parentTagId")]
    parent_tag_id: u32,
}

#[derive(Serialize)]
struct ItemTagArgs {
    #[serde(rename = "itemId")]
    item_id: u32,
    #[serde(rename = "tagId")]
    tag_id: u32,
}

#[derive(Serialize)]
struct TagTagArgs {
    #[serde(rename = "childTagId")]
    child_tag_id: u32,
    #[serde(rename = "parentTagId")]
    parent_tag_id: u32,
}

#[derive(Serialize)]
struct MoveTagArgs {
    id: u32,
    position: i32,
}

#[derive(Serialize)]
struct MoveChildTagArgs {
    #[serde(rename = "childTagId")]
    child_tag_id: u32,
    #[serde(rename = "parentTagId")]
    parent_tag_id: u32,
    position: i32,
}

// ========================
// Tag CRUD Commands
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

// ========================
// Item-Tag Relationship Commands
// ========================

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
// Tag-Tag Relationship Commands (Multi-parent)
// ========================

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
// Tag Movement Commands
// ========================

pub async fn move_tag(id: u32, position: i32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&MoveTagArgs { id, position }).map_err(|e| e.to_string())?;
    let _ = invoke("move_tag", js_args).await;
    Ok(())
}

pub async fn move_child_tag(child_tag_id: u32, parent_tag_id: u32, position: i32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&MoveChildTagArgs { child_tag_id, parent_tag_id, position }).map_err(|e| e.to_string())?;
    let _ = invoke("move_child_tag", js_args).await;
    Ok(())
}
