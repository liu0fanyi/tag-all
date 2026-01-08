//! Tauri Command Wrappers
//!
//! Frontend bindings to backend commands, organized by domain.

mod item;
mod tag;
mod workspace;
mod window;
mod sync;
mod files;
mod dialog;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

// Re-export all public items
pub use item::*;
pub use tag::*;
pub use workspace::*;
pub use window::*;
pub use sync::*;
pub use files::*;
pub use dialog::*;
