//! Tree Item Component
//!
//! Individual item in the tree view with right-click tagging and tag column.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::models::{Item, Tag};
use crate::commands;
use crate::context::AppContext;
use crate::components::EditTarget;

/// A single item row in the tree
#[component]
pub fn TreeItem(
    item: Item,
    depth: usize,
    has_children: bool,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    // Get context from parent
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
    let id = item.id;
    let completed = item.completed;
    let collapsed = item.collapsed;
    let text = item.text.clone();
    let position = item.position;
    let text_for_menu = text.clone();
    let indent = depth * 24;
    
    // Load tags for this item
    let (item_tags, set_item_tags) = signal(Vec::<Tag>::new());
    
    Effect::new(move |_| {
        let _ = ctx.reload_trigger.get();
        spawn_local(async move {
            if let Ok(tags) = commands::get_item_tags(id).await {
                set_item_tags.set(tags);
            }
        });
    });
    
    // Right-click handler
    let on_context_menu = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        set_editing_target.set(Some(EditTarget::Item(id, text_for_menu.clone())));
    };

    view! {
        <div
            class=move || if completed { "item-row completed" } else { "item-row" }
            style=format!("margin-left: {}px;", indent)
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
            
            // Checkbox
            <input
                type="checkbox"
                checked=completed
                on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                on:change=move |_| {
                    spawn_local(async move {
                        let _ = commands::toggle_item(id).await;
                        ctx.reload();
                    });
                }
            />
            
            // Text with position for debugging
            <span class="item-text">{format!("[{}] {}", position, text)}</span>
            
            // Add child button
            <button class="add-child-btn" on:click=move |ev| {
                ev.stop_propagation();
                ctx.set_adding_under(Some(id));
            }>"+"</button>
            
            // Delete button
            <button class="delete-btn" on:click=move |ev| {
                ev.stop_propagation();
                spawn_local(async move {
                    let _ = commands::delete_item(id).await;
                    ctx.reload();
                });
            }>"×"</button>
            
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
