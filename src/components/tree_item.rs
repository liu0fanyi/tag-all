//! Tree Item Component
//!
//! Individual item in the tree view with type-specific behavior.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::models::{Item, Tag};
use crate::commands;
use crate::context::AppContext;
use crate::components::{EditTarget, DeleteConfirmButton};

/// A single item row in the tree
#[component]
pub fn TreeItem(
    item: Item,
    depth: usize,
    has_children: bool,
    editing_target: ReadSignal<Option<EditTarget>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
    memo_editing_target: ReadSignal<Option<EditTarget>>,
    set_memo_editing_target: WriteSignal<Option<EditTarget>>,
    set_selected_item: WriteSignal<Option<u32>>,
) -> impl IntoView {
    // Get context from parent
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
    let id = item.id;
    let completed = item.completed;
    let collapsed = item.collapsed;
    let text = item.text.clone();
    let item_type = item.item_type.clone();
    let target_count = item.target_count;
    let current_count = item.current_count;
    let text_for_menu = text.clone();
    let indent = depth * 24;
    
    // Load tags for this item
    let (item_tags, set_item_tags) = signal(Vec::<Tag>::new());
    
    Effect::new(move |_| {
        let _ = ctx.reload_trigger.get();
        spawn_local(async move {
            if let Ok(tags) = commands::get_item_tags(id).await {
                // Backend sorts by pinyin
                set_item_tags.set(tags);
            }
        });
    });
    
    // Debounce for contextmenu to prevent duplicate events
    let (last_click_time, set_last_click_time) = signal(0f64);
    
    // Left-click handler - just select item (no editor)
    let on_click_for_editor = move |_| {
        set_selected_item.set(Some(id));
    };
    
    // Right-click handler - opens BOTH properties editor AND memo editor
    let text_for_click = text_for_menu.clone();
    let text_for_click2 = text_for_menu.clone();
    let on_context_menu = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        
        // Debounce: ignore events within 100ms
        let now = js_sys::Date::now();
        let last = last_click_time.get();
        if now - last < 100.0 {
            return;
        }
        set_last_click_time.set(now);
        
        set_selected_item.set(Some(id));
        
        // Check if already editing this item
        let current = editing_target.get();
        let is_editing_this = matches!(&current, Some(EditTarget::Item(eid, _)) if *eid == id);
        if is_editing_this {
            // Close both editors
            set_editing_target.set(None);
            set_memo_editing_target.set(None);
            
            // Shrink window
            spawn_local(async {
                let _ = commands::shrink_window(800, 700).await;
            });
        } else {
            // Open both editors
            set_editing_target.set(Some(EditTarget::Item(id, text_for_click.clone())));
            set_memo_editing_target.set(Some(EditTarget::Item(id, text_for_click2.clone())));
            
            // Expand window
            spawn_local(async {
                let _ = commands::resize_window(1800, 700).await;
            });
        }
    };
    
    // Type icon
    let type_icon = match item_type.as_str() {
        "daily" => "🔄",
        "once" => "✓",
        "countdown" => "⏳",
        "document" => "📑",
        _ => "📌",
    };
    
    // Check if should show checkbox
    let show_checkbox = item_type != "document";
    let is_countdown = item_type == "countdown";
    let is_once = item_type == "once";

    view! {
        <div
            class=move || if completed { "item-row completed" } else { "item-row" }
            style=format!("margin-left: {}px;", indent)
            on:click=on_click_for_editor
            on:contextmenu=on_context_menu
        >
            // Collapse toggle
            {if has_children {
                view! {
                    <button class="collapse-btn" on:click=move |ev| {
                        ev.stop_propagation();
                        spawn_local(async move {
                            let _ = commands::toggle_collapsed(id).await;
                            ctx.reload();
                        });
                    }>
                        {if collapsed { "▶" } else { "▼" }}
                    </button>
                }.into_any()
            } else {
                view! { <span class="collapse-placeholder">"·"</span> }.into_any()
            }}
            
            // Type icon
            <span class="type-icon" title=item_type.clone()>{type_icon}</span>
            
            // Checkbox / -1 button / nothing (based on type)
            {if !show_checkbox {
                // Document type - no checkbox
                view! { <span class="checkbox-placeholder"></span> }.into_any()
            } else if is_countdown {
                // Countdown type - always show -1 button (even when completed for resetting)
                view! {
                    <button 
                        class="decrement-btn" 
                        on:click=move |ev| {
                            ev.stop_propagation();
                            spawn_local(async move {
                                let _ = commands::decrement_item(id).await;
                                ctx.reload();
                            });
                        }
                    >
                        "-1"
                    </button>
                }.into_any()
            } else {
                // Regular checkbox
                view! {
                    <input
                        type="checkbox"
                        checked=completed
                        on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                        on:change=move |_| {
                            let is_once = is_once;
                            spawn_local(async move {
                                if is_once && !completed {
                                    // Once type - delete on complete
                                    let _ = commands::delete_item(id).await;
                                } else {
                                    let _ = commands::toggle_item(id).await;
                                }
                                ctx.reload();
                            });
                        }
                    />
                }.into_any()
            }}
            
            // Text
            <span class="item-text">{text}</span>
            
            // Countdown editable input (only for countdown type)
            {if is_countdown {
                // Calculate width based on digit count (minimum 2 chars)
                let char_count = current_count.to_string().len().max(2);
                let width_style = format!("width: {}ch;", char_count + 1);
                
                view! { 
                    <input
                        type="number"
                        class="countdown-inline-input"
                        style=width_style
                        prop:value=current_count
                        on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                        on:input=move |ev: web_sys::Event| {
                            use wasm_bindgen::JsCast;
                            let target = ev.target().unwrap();
                            let input = target.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                            // Adjust width based on content
                            let len = input.value().len().max(2);
                            let _ = input.set_attribute("style", &format!("width: {}ch;", len + 1));
                        }
                        on:change=move |ev| {
                            use wasm_bindgen::JsCast;
                            ev.stop_propagation();
                            let target = ev.target().unwrap();
                            let input = target.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                            let value: i32 = input.value().parse().unwrap_or(0);
                            spawn_local(async move {
                                let _ = commands::set_item_count(id, Some(value)).await;
                                ctx.reload();
                            });
                        }
                    />
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            
            // Add child button
            <button class="add-child-btn" on:click=move |ev| {
                ev.stop_propagation();
                ctx.set_adding_under(Some(id));
            }>"+"</button>
            
            // Delete button with confirmation
            <DeleteConfirmButton
                button_class="delete-btn"
                on_confirm=move || {
                    spawn_local(async move {
                        let _ = commands::delete_item(id).await;
                        ctx.reload();
                    });
                }
            />
            
            // Tags column (right of delete)
            <div class="item-tags-column">
                <For
                    each=move || item_tags.get()
                    key=|tag| tag.id
                    children=move |tag| {
                        let color = tag.color.clone().unwrap_or_else(|| "#666".to_string());
                        view! {
                            <span
                                class="item-tag-chip"
                                style=format!("background-color: {};", color)
                                title=tag.name.clone()
                            >
                                {tag.name.chars().next().unwrap_or('?')}
                            </span>
                        }
                    }
                />
            </div>
        </div>
    }
}
