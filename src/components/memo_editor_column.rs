//! Memo Editor Column
//!
//! Fourth column for editing item/tag memo with side-by-side edit and preview.

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ClipboardEvent, DataTransferItem, DataTransferItemList, File, FileReader};

use crate::commands;
use crate::components::EditTarget;
use crate::markdown::{parse_markdown, trigger_math_render};

/// Memo editor column with side-by-side edit and preview
#[component]
pub fn MemoEditorColumn(
    editing_target: ReadSignal<Option<EditTarget>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let (memo_content, set_memo_content) = signal(String::new());
    let (last_target_id, set_last_target_id) = signal::<Option<u32>>(None);
    
    // Load memo when target changes
    Effect::new(move |_| {
        if let Some(target) = editing_target.get() {
            let current_id = match &target {
                EditTarget::Item(id, _) => Some(*id),
                _ => None,
            };
            
            // Only reload if target changed
            if current_id != last_target_id.get() {
                set_last_target_id.set(current_id);
                
                if let EditTarget::Item(id, _) = &target {
                    let id = *id;
                    spawn_local(async move {
                        if let Ok(Some(item)) = commands::get_item(id).await {
                            set_memo_content.set(item.memo.unwrap_or_default());
                        }
                    });
                }
            }
        } else {
            set_last_target_id.set(None);
        }
    });
    
    // Save memo on blur
    let save_memo = move || {
        if let Some(target) = editing_target.get() {
            if let EditTarget::Item(id, _) = target {
                let content = memo_content.get();
                let memo = if content.is_empty() { None } else { Some(content) };
                spawn_local(async move {
                    let _ = commands::update_item_memo(id, memo.as_deref()).await;
                });
            }
        }
    };
    
    // Handle paste event for images
    let handle_paste = move |ev: ClipboardEvent| {
        if let Some(data_transfer) = ev.clipboard_data() {
            let items = data_transfer.items();
            
            for i in 0..items.length() {
                if let Some(item) = items.get(i) {
                    let item_type = item.type_();
                    
                    // Check if it's an image
                    if item_type.starts_with("image/") {
                        ev.prevent_default();
                        
                        // Get the textarea element for later use
                        let textarea_opt: Option<web_sys::HtmlTextAreaElement> = ev.target()
                            .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok());
                        
                        if let Ok(Some(file)) = item.get_as_file() {
                            let set_memo = set_memo_content.clone();
                            
                            // Read file as base64
                            let reader = FileReader::new().unwrap();
                            let reader_clone = reader.clone();
                            
                            let onload = Closure::once(Box::new(move |_: web_sys::Event| {
                                if let Ok(result) = reader_clone.result() {
                                    if let Some(data_url) = result.as_string() {
                                        let textarea_for_async = textarea_opt.clone();
                                        // Call backend to save image
                                        spawn_local(async move {
                                            match commands::save_clipboard_image(&data_url).await {
                                                Ok(path) => {
                                                    let img_markdown = format!("![]({})", path);
                                                    
                                                    // Use execCommand to insert text at cursor with native undo support
                                                    if let Some(textarea) = textarea_for_async {
                                                        // Focus the textarea
                                                        let _ = textarea.focus();
                                                        
                                                        // Call document.execCommand via js_sys
                                                        if let Some(window) = web_sys::window() {
                                                            if let Some(document) = window.document() {
                                                                // Use js_sys to call execCommand dynamically
                                                                let exec_command = js_sys::Reflect::get(
                                                                    &document,
                                                                    &wasm_bindgen::JsValue::from_str("execCommand")
                                                                );
                                                                if let Ok(func) = exec_command {
                                                                    if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                                                                        let _ = func.call3(
                                                                            &document,
                                                                            &wasm_bindgen::JsValue::from_str("insertText"),
                                                                            &wasm_bindgen::JsValue::FALSE,
                                                                            &wasm_bindgen::JsValue::from_str(&img_markdown)
                                                                        );
                                                                    }
                                                                }
                                                                // Update our signal to match textarea value
                                                                set_memo.set(textarea.value());
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    web_sys::console::error_1(&format!("Failed to save image: {}", e).into());
                                                }
                                            }
                                        });
                                    }
                                }
                            }) as Box<dyn FnOnce(_)>);
                            
                            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                            onload.forget();
                            
                            let _ = reader.read_as_data_url(&file);
                        }
                        break;
                    }
                }
            }
        }
    };
    
    // Get title
    let title = move || {
        match editing_target.get() {
            Some(EditTarget::Item(_, name)) => format!("备注: {}", name),
            Some(EditTarget::Tag(_, name)) => format!("备注: {}", name),
            Some(EditTarget::MultiItems(_)) => String::new(), // Not shown for multi-items
            None => String::new(),
        }
    };
    
    // Rendered HTML for preview
    let rendered_html = move || {
        let html = parse_markdown(&memo_content.get());
        trigger_math_render("#memo-preview"); // Scope to this element
        html
    };
    
    // Vim mode indicator
    let (vim_mode, set_vim_mode) = signal(String::from("insert"));
    
    view! {
        <Show when=move || editing_target.get().is_some()>
            <div class="memo-editor-column">
                <div class="memo-editor-header">
                    <span class="memo-editor-title">{title}</span>
                    <span class=move || format!("vim-mode-indicator {}", vim_mode.get())>{move || vim_mode.get()}</span>
                    <button class="close-btn" on:click=move |_| set_editing_target.set(None)>"×"</button>
                </div>
                
                <div class="memo-editor-body">
                    // Left: Edit area
                    <div class="memo-edit-pane">
                        <div class="pane-header">"编辑"</div>
                        <textarea
                            class="memo-textarea"
                            prop:value=move || memo_content.get()
                            on:input=move |ev| {
                                let target = ev.target().unwrap();
                                let textarea = target.dyn_ref::<web_sys::HtmlTextAreaElement>().unwrap();
                                set_memo_content.set(textarea.value());
                            }
                            on:vimmodechange=move |ev: web_sys::CustomEvent| {
                                if let Some(mode) = js_sys::Reflect::get(&ev.detail(), &wasm_bindgen::JsValue::from_str("mode")).ok() {
                                    if let Some(mode_str) = mode.as_string() {
                                        set_vim_mode.set(mode_str);
                                    }
                                }
                            }
                            on:paste=handle_paste
                            on:blur=move |_| save_memo()
                            placeholder="输入 Markdown 内容... (按 Esc 进入 Normal 模式)"
                        ></textarea>
                    </div>
                    
                    // Right: Preview area
                    <div class="memo-preview-pane">
                        <div class="pane-header">"预览"</div>
                        <div id="memo-preview" class="memo-preview-content" inner_html=rendered_html></div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
