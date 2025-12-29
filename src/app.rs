//! Tag-All Frontend App
//!
//! Main application component with three-column layout.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::models::{Item, Tag};
use crate::commands;
use crate::context::AppContext;
use crate::components::{NewItemForm, TagColumn, TagEditor, ItemTreeView, EditTarget};

#[component]
pub fn App() -> impl IntoView {
    // State
    let (items, set_items) = signal(Vec::<Item>::new());
    let (tags, set_tags) = signal(Vec::<Tag>::new());
    let (adding_under, set_adding_under) = signal::<Option<u32>>(None);
    let (reload_trigger, set_reload_trigger) = signal(0u32);
    let (selected_item, set_selected_item) = signal::<Option<u32>>(None);
    let (selected_tag, set_selected_tag) = signal::<Option<u32>>(None);
    let (editing_target, set_editing_target) = signal::<Option<EditTarget>>(None);

    // Provide context to all children
    provide_context(AppContext::new((reload_trigger, set_reload_trigger), (adding_under, set_adding_under)));

    // Load items and tags on mount and when trigger changes
    Effect::new(move |_| {
        let trigger = reload_trigger.get();
        web_sys::console::log_1(&format!("[APP] Effect running, trigger={}", trigger).into());
        spawn_local(async move {
            if let Ok(loaded) = commands::list_items().await {
                web_sys::console::log_1(&format!("[APP] Loaded {} items", loaded.len()).into());
                set_items.set(loaded);
            }
            if let Ok(loaded) = commands::list_tags().await {
                set_tags.set(loaded);
            }
        });
    });

    view! {
        <div class="app-layout">
            // Left: Tag Column
            <TagColumn
                selected_tag=selected_tag
                set_selected_tag=set_selected_tag
                set_editing_target=set_editing_target
            />
            
            // Center: Main Content
            <main class="main-content">
                <h1>"Tag-All"</h1>
                
                <NewItemForm />
                
                <ItemTreeView
                    items=items
                    selected_item=selected_item
                    set_selected_item=set_selected_item
                    set_editing_target=set_editing_target
                />
                
                <p class="item-count">{move || format!("{} items, {} tags", items.get().len(), tags.get().len())}</p>
            </main>
            
            // Right: Tag Editor (third column, shown when editing)
            <TagEditor
                editing_target=editing_target
                set_editing_target=set_editing_target
            />
        </div>
    }
}
