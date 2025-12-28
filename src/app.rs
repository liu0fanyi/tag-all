//! Tag-All Frontend App
//!
//! Main application component.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::models::Item;
use crate::commands;
use crate::tree::flatten_tree;
use crate::context::AppContext;
use crate::components::{TreeItem, NewItemForm};

#[component]
pub fn App() -> impl IntoView {
    // State
    let (items, set_items) = signal(Vec::<Item>::new());
    let (adding_under, set_adding_under) = signal::<Option<u32>>(None);
    let (reload_trigger, set_reload_trigger) = signal(0u32);

    // Provide context to all children
    provide_context(AppContext::new(set_reload_trigger, (adding_under, set_adding_under)));

    // Load items on mount and when trigger changes
    Effect::new(move |_| {
        let _ = reload_trigger.get();
        spawn_local(async move {
            if let Ok(loaded) = commands::list_items().await {
                set_items.set(loaded);
            }
        });
    });

    // Computed tree
    let tree_items = move || flatten_tree(&items.get());

    view! {
        <main class="container">
            <h1>"Tag-All Todo"</h1>
            
            <NewItemForm />
            
            <div class="tree-view">
                <For
                    each=tree_items
                    key=|(item, _)| item.id
                    children=move |(item, depth)| {
                        let has_children = items.get().iter().any(|i| i.parent_id == Some(item.id));
                        
                        view! {
                            <TreeItem
                                item=item
                                depth=depth
                                has_children=has_children
                            />
                        }
                    }
                />
            </div>
            
            <p class="item-count">{move || format!("{} items", items.get().len())}</p>
        </main>
    }
}
