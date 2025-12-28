//! Item Tree View Component
//!
//! Displays items in a tree structure.

use leptos::prelude::*;

use crate::models::Item;
use crate::tree::flatten_tree;
use crate::components::{TreeItem, EditTarget};

/// Item tree view component
#[component]
pub fn ItemTreeView(
    items: ReadSignal<Vec<Item>>,
    selected_item: ReadSignal<Option<u32>>,
    set_selected_item: WriteSignal<Option<u32>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let tree_items = move || flatten_tree(&items.get());

    view! {
        <div class="tree-view">
            <For
                each=tree_items
                key=|(item, _)| item.id
                children=move |(item, depth)| {
                    let id = item.id;
                    let has_children = items.get().iter().any(|i| i.parent_id == Some(item.id));
                    let is_selected = move || selected_item.get() == Some(id);
                    
                    view! {
                        <div
                            class=move || if is_selected() { "tree-item-wrapper selected" } else { "tree-item-wrapper" }
                            on:click=move |_| set_selected_item.set(Some(id))
                        >
                            <TreeItem
                                item=item.clone()
                                depth=depth
                                has_children=has_children
                                set_editing_target=set_editing_target
                            />
                        </div>
                    }
                }
            />
        </div>
    }
}
