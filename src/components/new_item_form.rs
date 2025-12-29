//! New Item Form Component
//!
//! Form for creating new items with type selector.

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::commands::{self, CreateItemArgs};
use crate::context::AppContext;

/// Item type options
const ITEM_TYPES: &[(&str, &str)] = &[
    ("daily", "循环"),
    ("once", "一次"),
    ("countdown", "倒数"),
    ("document", "文档"),
];

/// Form for creating new items (root or child)
#[component]
pub fn NewItemForm() -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
    let (new_text, set_new_text) = signal(String::new());
    let (item_type, set_item_type) = signal(String::from("daily"));

    let create_item = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let text = new_text.get();
        if text.is_empty() { return; }
        let parent = ctx.adding_under.get();
        let workspace = ctx.current_workspace.get();
        let selected_type = item_type.get();
        
        spawn_local(async move {
            let args = CreateItemArgs {
                text: &text,
                item_type: Some(&selected_type),
                parent_id: parent,
                workspace_id: Some(workspace),
            };
            if commands::create_item(&args).await.is_ok() {
                set_new_text.set(String::new());
                ctx.set_adding_under(None);
                ctx.reload();
            }
        });
    };

    view! {
        <form class="new-item-form" on:submit=create_item>
            <div class="new-item-row">
                <input
                    type="text"
                    placeholder=move || {
                        if let Some(pid) = ctx.adding_under.get() {
                            format!("Add child under #{}...", pid)
                        } else {
                            "Add new item...".to_string()
                        }
                    }
                    prop:value=move || new_text.get()
                    on:input=move |ev| {
                        let target = ev.target().unwrap();
                        let input = target.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                        set_new_text.set(input.value());
                    }
                />
                <button type="submit">"Add"</button>
            </div>
            
            <div class="type-selector-row">
                {ITEM_TYPES.iter().map(|(value, label)| {
                    let val = value.to_string();
                    let val_clone = val.clone();
                    let is_selected = move || item_type.get() == val;
                    view! {
                        <button
                            type="button"
                            class=move || if is_selected() { "type-btn small active" } else { "type-btn small" }
                            on:click=move |_| set_item_type.set(val_clone.clone())
                        >
                            {*label}
                        </button>
                    }
                }).collect_view()}
            </div>
            
            {move || ctx.adding_under.get().map(|pid| view! {
                <button type="button" class="cancel-btn" on:click=move |_| ctx.set_adding_under(None)>
                    "Cancel (#" {pid} ")"
                </button>
            })}
        </form>
    }
}
