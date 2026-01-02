//! Cloud Sync Command Wrappers
//!
//! Frontend bindings for cloud synchronization commands.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

/// Sync configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub url: String,
    pub token: String,
}

/// Configure cloud synchronization
pub async fn configure_cloud_sync(url: String, token: String) -> Result<(), String> {
    #[derive(Serialize)]
    struct Args {
        url: String,
        token: String,
    }
    
    let args = serde_wasm_bindgen::to_value(&Args { url, token })
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    let result = invoke("configure_cloud_sync", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Response error: {}", e))
}

/// Get current sync configuration
pub async fn get_cloud_sync_config() -> Result<Option<SyncConfig>, String> {
    let result = invoke("get_cloud_sync_config", JsValue::NULL).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Response error: {}", e))
}

/// Manually trigger database sync
pub async fn sync_cloud_db() -> Result<(), String> {
    let result = invoke("sync_cloud_db", JsValue::NULL).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Response error: {}", e))
}
