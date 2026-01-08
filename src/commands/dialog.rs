use wasm_bindgen::prelude::*;
use crate::commands::invoke;

pub async fn pick_folder() -> Result<Option<String>, String> {
    let result = invoke("pick_folder", JsValue::NULL).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}
