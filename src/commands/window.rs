//! Window State Commands
//!
//! Frontend bindings for window state persistence and resizing.

use wasm_bindgen::prelude::*;
use serde::Serialize;
use super::invoke;

// ========================
// Types
// ========================

#[derive(Serialize)]
struct WindowStateArgs {
    width: f64,
    height: f64,
    x: f64,
    y: f64,
    pinned: bool,
}

#[derive(Serialize)]
struct ResizeArgs {
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WindowState {
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
    pub pinned: bool,
}

// ========================
// Commands
// ========================

pub async fn save_window_state(width: f64, height: f64, x: f64, y: f64, pinned: bool) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&WindowStateArgs { width, height, x, y, pinned }).map_err(|e| e.to_string())?;
    let _ = invoke("save_window_state", js_args).await;
    Ok(())
}

pub async fn load_window_state() -> Result<Option<WindowState>, String> {
    let result = invoke("load_window_state", JsValue::NULL).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

/// Resize window to fit content (only expands, never shrinks)
pub async fn resize_window(width: u32, height: u32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&ResizeArgs { width, height }).map_err(|e| e.to_string())?;
    let _ = invoke("resize_window", js_args).await;
    Ok(())
}

/// Shrink window to specified size (allows reducing)
pub async fn shrink_window(width: u32, height: u32) -> Result<(), String> {
    let js_args = serde_wasm_bindgen::to_value(&ResizeArgs { width, height }).map_err(|e| e.to_string())?;
    let _ = invoke("shrink_window", js_args).await;
    Ok(())
}
