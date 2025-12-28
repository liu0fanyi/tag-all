//! Tree Item Component
//!
//! Individual item in the tree view.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::models::Item;
use crate::commands;
use crate::context::AppContext;

/// A single item row in the tree
#[component]
pub fn TreeItem(
    item: Item,
    depth: usize,
    has_children: bool,
) -> impl IntoView {
    // Get context from parent
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
    let id = item.id;
    let completed = item.completed;
    let collapsed = item.collapsed;
    let text = item.text.clone();
    let indent = depth * 24;

    view! {
        <div
            class=move || if completed { "item-row completed" } else { "item-row" }
            style=format!("margin-left: {}px;", indent)
        >
            // Collapse toggle
            {if has_children {
                view! {
                    <button class="collapse-btn" on:click=move |_| {
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
                on:change=move |_| {
                    spawn_local(async move {
                        let _ = commands::toggle_item(id).await;
                        ctx.reload();
                    });
                }
            />
            
            // Text
            <span class="item-text">{text}</span>
            
            // Add child button
            <button class="add-child-btn" on:click=move |_| ctx.set_adding_under(Some(id))>"+"</button>
            
            // Delete button
            <button class="delete-btn" on:click=move |_| {
                spawn_local(async move {
                    let _ = commands::delete_item(id).await;
                    ctx.reload();
                });
            }>"×"</button>
        </div>
    }
}
