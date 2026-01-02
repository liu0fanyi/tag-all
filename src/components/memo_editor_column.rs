//! Memo Editor Column
//!
//! Fourth column for editing item/tag memo with side-by-side edit and preview.

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::commands;
use crate::components::EditTarget;

/// Simple Markdown to HTML conversion
fn markdown_to_html(md: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut in_list = false;
    
    for line in md.lines() {
        // Code blocks
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>");
                in_code_block = false;
            } else {
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }
        
        if in_code_block {
            html.push_str(&escape_html(line));
            html.push('\n');
            continue;
        }
        
        // Headers
        if line.starts_with("### ") {
            html.push_str(&format!("<h3>{}</h3>", escape_html(&line[4..])));
            continue;
        }
        if line.starts_with("## ") {
            html.push_str(&format!("<h2>{}</h2>", escape_html(&line[3..])));
            continue;
        }
        if line.starts_with("# ") {
            html.push_str(&format!("<h1>{}</h1>", escape_html(&line[2..])));
            continue;
        }
        
        // Lists
        if line.starts_with("- ") || line.starts_with("* ") {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            html.push_str(&format!("<li>{}</li>", escape_html(&line[2..])));
            continue;
        } else if in_list {
            html.push_str("</ul>");
            in_list = false;
        }
        
        // Empty line
        if line.trim().is_empty() {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            continue;
        }
        
        // Regular paragraph
        html.push_str(&format!("<p>{}</p>", escape_html(line)));
    }
    
    if in_list {
        html.push_str("</ul>");
    }
    if in_code_block {
        html.push_str("</code></pre>");
    }
    
    html
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

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
    let rendered_html = move || markdown_to_html(&memo_content.get());
    
    view! {
        <Show when=move || editing_target.get().is_some()>
            <div class="memo-editor-column">
                <div class="memo-editor-header">
                    <span class="memo-editor-title">{title}</span>
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
                            on:blur=move |_| save_memo()
                            placeholder="输入 Markdown 内容..."
                        ></textarea>
                    </div>
                    
                    // Right: Preview area
                    <div class="memo-preview-pane">
                        <div class="pane-header">"预览"</div>
                        <div class="memo-preview-content" inner_html=rendered_html></div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
