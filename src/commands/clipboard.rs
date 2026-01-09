//! Clipboard Commands
//!
//! Frontend wrappers for clipboard operations.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize)]
struct SaveClipboardImageArgs {
    data: String,
}

/// Save clipboard image data and return the file path
pub async fn save_clipboard_image(data: &str) -> Result<String, String> {
    let args = SaveClipboardImageArgs {
        data: data.to_string(),
    };
    
    let args_js = serde_wasm_bindgen::to_value(&args)
        .map_err(|e| format!("Failed to serialize args: {}", e))?;
    
    let result = invoke("save_clipboard_image", args_js).await;
    
    serde_wasm_bindgen::from_value::<String>(result)
        .map_err(|e| format!("Failed to parse result: {}", e))
}

/// Clean up unused assets
pub async fn clean_unused_assets() -> Result<usize, String> {
    let result = invoke("clean_unused_assets", JsValue::NULL).await;
    
    serde_wasm_bindgen::from_value::<usize>(result)
        .map_err(|e| format!("Failed to parse result: {}", e))
}
