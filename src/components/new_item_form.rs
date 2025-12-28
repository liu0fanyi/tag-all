//! New Item Form Component
//!
//! Form for creating new items (not tags).

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::commands::{self, CreateItemArgs};
use crate::context::AppContext;

/// Form for creating new items (root or child)
#[component]
pub fn NewItemForm() -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
    let (new_text, set_new_text) = signal(String::new());

    let create_item = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let text = new_text.get();
        if text.is_empty() { return; }
        let parent = ctx.adding_under.get();
        
        spawn_local(async move {
            let args = CreateItemArgs {
                text: &text,
                item_type: Some("daily"),
                parent_id: parent,
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
            {move || ctx.adding_under.get().map(|pid| view! {
                <button type="button" on:click=move |_| ctx.set_adding_under(None)>
                    "Cancel (#" {pid} ")"
                </button>
            })}
        </form>
    }
}
