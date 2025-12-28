//! Tag-All Frontend App
//!
//! Main application component for the tag-all todo list.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

/// Item data structure (matches backend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub text: String,
    pub completed: bool,
    pub item_type: String,
    pub memo: Option<String>,
    pub target_count: Option<i32>,
    pub current_count: i32,
}

#[derive(Serialize)]
struct CreateItemArgs<'a> {
    text: &'a str,
    #[serde(rename = "itemType")]
    item_type: Option<&'a str>,
}

#[component]
pub fn App() -> impl IntoView {
    // State: list of items
    let (items, set_items) = signal(Vec::<Item>::new());
    // State: new item input
    let (new_text, set_new_text) = signal(String::new());

    // Load items on mount
    Effect::new(move |_| {
        spawn_local(async move {
            let result = invoke("list_items", JsValue::NULL).await;
            if let Ok(loaded) = serde_wasm_bindgen::from_value::<Vec<Item>>(result) {
                set_items.set(loaded);
            }
        });
    });

    // Create new item
    let create_item = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let text = new_text.get();
        if text.is_empty() {
            return;
        }
        
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&CreateItemArgs {
                text: &text,
                item_type: Some("daily"),
            }).unwrap();
            
            let result = invoke("create_item", args).await;
            if let Ok(item) = serde_wasm_bindgen::from_value::<Item>(result) {
                set_items.update(|list| list.push(item));
                set_new_text.set(String::new());
            }
        });
    };

    // Toggle item completion
    let toggle_item = move |id: u32| {
        spawn_local(async move {
            #[derive(Serialize)]
            struct ToggleArgs { id: u32 }
            
            let args = serde_wasm_bindgen::to_value(&ToggleArgs { id }).unwrap();
            let result = invoke("toggle_item", args).await;
            
            if let Ok(updated) = serde_wasm_bindgen::from_value::<Item>(result) {
                set_items.update(|list| {
                    if let Some(item) = list.iter_mut().find(|i| i.id == id) {
                        item.completed = updated.completed;
                    }
                });
            }
        });
    };

    // Delete item
    let delete_item = move |id: u32| {
        spawn_local(async move {
            #[derive(Serialize)]
            struct DeleteArgs { id: u32 }
            
            let args = serde_wasm_bindgen::to_value(&DeleteArgs { id }).unwrap();
            invoke("delete_item", args).await;
            
            set_items.update(|list| list.retain(|i| i.id != id));
        });
    };

    view! {
        <main class="container">
            <h1>"Tag-All Todo"</h1>
            
            // New item form
            <form class="new-item-form" on:submit=create_item>
                <input
                    type="text"
                    placeholder="Add new item..."
                    prop:value=move || new_text.get()
                    on:input=move |ev| {
                        use wasm_bindgen::JsCast;
                        let target = ev.target().unwrap();
                        let input = target.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                        set_new_text.set(input.value());
                    }
                />
                <button type="submit">"Add"</button>
            </form>
            
            // Item list
            <ul class="item-list">
                <For
                    each=move || items.get()
                    key=|item| item.id
                    children=move |item| {
                        let id = item.id;
                        let completed = item.completed;
                        
                        view! {
                            <li class=move || if completed { "completed" } else { "" }>
                                <input
                                    type="checkbox"
                                    checked=completed
                                    on:change=move |_| toggle_item(id)
                                />
                                <span>{item.text.clone()}</span>
                                <button
                                    class="delete-btn"
                                    on:click=move |_| delete_item(id)
                                >"×"</button>
                            </li>
                        }
                    }
                />
            </ul>
            
            <p class="item-count">{move || format!("{} items", items.get().len())}</p>
        </main>
    }
}
