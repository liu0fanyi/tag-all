//! File Commands
//!
//! Frontend bindings for file-related backend commands.

use wasm_bindgen::prelude::*;
use serde::Serialize;
use crate::models::{FileViewItem, Item};
use super::invoke;

#[derive(Serialize)]
struct ListDirectoryArgs<'a> {
    path: &'a str,
}

#[derive(Serialize)]
struct EnsureFileItemArgs<'a> {
    path: &'a str,
}

#[derive(Serialize)]
struct OpenFileArgs<'a> {
    path: &'a str,
}

pub async fn list_directory(path: &str) -> Result<Vec<FileViewItem>, String> {
    let js_args = serde_wasm_bindgen::to_value(&ListDirectoryArgs { path }).map_err(|e| e.to_string())?;
    let result = invoke("list_directory", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn ensure_file_item(path: &str) -> Result<Item, String> {
    let js_args = serde_wasm_bindgen::to_value(&EnsureFileItemArgs { path }).map_err(|e| e.to_string())?;
    let result = invoke("ensure_file_item", js_args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

pub async fn open_file(path: &str) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&OpenFileArgs { path }).map_err(|e| e.to_string())?;
    invoke("open_file", js_args).await;
    Ok(())
}
