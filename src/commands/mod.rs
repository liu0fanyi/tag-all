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
mod clipboard;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> JsValue;
}

// Re-export commonly used items for convenience
pub use wasm_bindgen::prelude::JsValue;

/// Safe listen wrapper for event listening
pub async fn listen_safe<F>(event: &str, handler: F) -> Result<JsValue, String>
where
    F: FnMut(JsValue) + 'static,
{
    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(JsValue)>);
    // We intentionally leak the closure because the event listener needs to live as long as the app
    // or until manual unlistening (not implemented here for simplicity)
    let handler_ref = &closure;
    let result = listen(event, handler_ref).await;
    closure.forget();
    Ok(result)
}

// Re-export all public items
pub use item::*;
pub use tag::*;
pub use workspace::*;
pub use window::*;
pub use sync::*;
pub use files::*;
pub use dialog::*;
pub use clipboard::*;
