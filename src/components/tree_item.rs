//! Tree Item Component
//!
//! Individual item in the tree view with type-specific behavior.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::models::{Item, Tag};
use crate::commands;
use crate::context::AppContext;
use crate::components::{EditTarget, DeleteConfirmButton};
use crate::store::{use_app_store, store_update_item, store_remove_item, AppStateStoreFields};
use crate::markdown::parse_markdown_inline;

/// A single item row in the tree
#[component]
pub fn TreeItem(
    item: Item,
    depth: usize,
    has_children: bool,
    visible_item_ids: Memo<Vec<u32>>,
    selected_item: ReadSignal<Option<u32>>,
    selected_items: ReadSignal<Vec<u32>>,
    set_selected_items: WriteSignal<Vec<u32>>,
    editing_target: ReadSignal<Option<EditTarget>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
    memo_editing_target: ReadSignal<Option<EditTarget>>,
    set_memo_editing_target: WriteSignal<Option<EditTarget>>,
    set_selected_item: WriteSignal<Option<u32>>,
) -> impl IntoView {
    // Get context from parent
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    let store = use_app_store();
    
    let id = item.id;
    let position = item.position;
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
        // Watch store.tags_relation_version for changes
        let _ = store.tags_relation_version().get();
        spawn_local(async move {
            if let Ok(tags) = commands::get_item_tags(id).await {
                // Backend sorts by pinyin
                set_item_tags.set(tags);
            }
        });
    });
    
    // Debounce for contextmenu to prevent duplicate events
    let (last_click_time, set_last_click_time) = signal(0f64);
    
    // Left-click handler - standard multi-select behavior:
    // - Shift-click: select range from anchor (selected_item) to clicked item
    // - Ctrl-click: toggle individual item in selection
    // - Normal click: single select, clear multi-select
    let on_click_for_editor = move |ev: web_sys::MouseEvent| {
        if ev.shift_key() {
            // Shift-click: range selection from anchor to this item
            let anchor = selected_item.get();
            if let Some(anchor_id) = anchor {
                let ids = visible_item_ids.get();
                let anchor_idx = ids.iter().position(|&x| x == anchor_id);
                let current_idx = ids.iter().position(|&x| x == id);
                
                if let (Some(start), Some(end)) = (anchor_idx, current_idx) {
                    let (from, to) = if start <= end { (start, end) } else { (end, start) };
                    let range_ids: Vec<u32> = ids[from..=to].to_vec();
                    set_selected_items.set(range_ids);
                }
            } else {
                // No anchor yet, just select this item as anchor and add to selection
                set_selected_item.set(Some(id));
                set_selected_items.set(vec![id]);
            }
        } else if ev.ctrl_key() || ev.meta_key() {
            // Ctrl-click (or Cmd on Mac): toggle individual item in selection
            let anchor = selected_item.get();
            set_selected_items.update(|items| {
                // If starting multi-select from a single-selected item, include the anchor first
                if items.is_empty() {
                    if let Some(anchor_id) = anchor {
                        if anchor_id != id {
                            items.push(anchor_id);
                        }
                    }
                }
                // Toggle the clicked item
                if items.contains(&id) {
                    items.retain(|&x| x != id);
                } else {
                    items.push(id);
                }
            });
            // Update anchor to this item
            set_selected_item.set(Some(id));
        } else {
            // Normal click: single select, clear multi-select
            set_selected_item.set(Some(id));
            set_selected_items.set(Vec::new());
        }
    };
    
    // Right-click handler - opens editors based on selection mode
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
        
        let current_selected = selected_items.get();
        
        // Check if in multi-select mode (more than 1 item selected)
        if current_selected.len() > 1 {
            // Multi-select mode: only open TagEditor with MultiItems target
            let mut all_selected = current_selected.clone();
            if !all_selected.contains(&id) {
                all_selected.push(id);
                set_selected_items.set(all_selected.clone());
            }
            
            // Check if already editing these items
            let current_target = editing_target.get();
            let is_editing_multi = matches!(&current_target, Some(EditTarget::MultiItems(ids)) if ids == &all_selected);
            
            if is_editing_multi {
                // Close editor
                set_editing_target.set(None);
                set_memo_editing_target.set(None);
                // Do not shrink window on close
                spawn_local(async {
                    let _ = commands::shrink_window(800, 700).await;
                });
            } else {
                // Open TagEditor only (no MemoEditor for multi-select)
                set_editing_target.set(Some(EditTarget::MultiItems(all_selected)));
                set_memo_editing_target.set(None);
                // Smaller window since no memo editor
                spawn_local(async {
                    let _ = commands::resize_window(1200, 700).await;
                });
            }
        } else {
            // Single-select mode: original behavior
            set_selected_item.set(Some(id));
            set_selected_items.set(Vec::new());
            
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
                            if let Ok(new_collapsed) = commands::toggle_collapsed(id).await {
                                // Update only the collapsed field in store
                                store.items().write().iter_mut()
                                    .find(|i| i.id == id)
                                    .map(|i| i.collapsed = new_collapsed);
                            }
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
                                if let Ok(updated) = commands::decrement_item(id).await {
                                    store_update_item(&store, updated);
                                }
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
                                    store_remove_item(&store, id);
                                } else {
                                    if let Ok(updated) = commands::toggle_item(id).await {
                                        store_update_item(&store, updated);
                                    }
                                }
                            });
                        }
                    />
                }.into_any()
            }}
            
            // Text with position (supports color syntax like %r%red text%r%)
            {
                let formatted_text = format!("[{}] {}", position, text);
                let html_text = parse_markdown_inline(&formatted_text);
                view! {
                    <span class="item-text" inner_html=html_text></span>
                }
            }
            
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
                                if let Ok(updated) = commands::set_item_count(id, Some(value)).await {
                                    store_update_item(&store, updated);
                                }
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
                        store_remove_item(&store, id);
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
