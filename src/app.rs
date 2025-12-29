//! Tag-All Frontend App
//!
//! Main application component with three-column layout.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::models::{Item, Tag, Workspace};
use crate::commands;
use crate::context::AppContext;
use crate::components::{NewItemForm, TagColumn, TagEditor, ItemTreeView, EditTarget, WorkspaceTabBar};

#[component]
pub fn App() -> impl IntoView {
    // State
    let (items, set_items) = signal(Vec::<Item>::new());
    let (tags, set_tags) = signal(Vec::<Tag>::new());
    let (workspaces, set_workspaces) = signal(Vec::<Workspace>::new());
    let (current_workspace, set_current_workspace) = signal(1u32); // Default workspace ID = 1
    let (adding_under, set_adding_under) = signal::<Option<u32>>(None);
    let (reload_trigger, set_reload_trigger) = signal(0u32);
    let (selected_item, set_selected_item) = signal::<Option<u32>>(None);
    let (selected_tag, set_selected_tag) = signal::<Option<u32>>(None);
    let (editing_target, set_editing_target) = signal::<Option<EditTarget>>(None);

    // Provide context to all children
    provide_context(AppContext::new((reload_trigger, set_reload_trigger), (adding_under, set_adding_under), current_workspace));

    // Load workspaces on mount
    Effect::new(move |_| {
        let _ = reload_trigger.get();
        spawn_local(async move {
            if let Ok(loaded) = commands::list_workspaces().await {
                set_workspaces.set(loaded);
            }
        });
    });

    // Load items when workspace or trigger changes
    Effect::new(move |_| {
        let trigger = reload_trigger.get();
        let ws_id = current_workspace.get();
        web_sys::console::log_1(&format!("[APP] Loading items for workspace {}, trigger={}", ws_id, trigger).into());
        spawn_local(async move {
            if let Ok(loaded) = commands::list_items_by_workspace(ws_id).await {
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
                // Workspace Tab Bar
                <WorkspaceTabBar
                    workspaces=workspaces
                    current_workspace=current_workspace
                    set_current_workspace=set_current_workspace
                />
                
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
