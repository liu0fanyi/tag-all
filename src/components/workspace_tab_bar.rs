//! Workspace Tab Bar Component
//!
//! Tab bar for switching between workspaces in the main content area.

use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::models::Workspace;
use crate::commands;
use crate::context::AppContext;
use crate::store::{use_app_store, AppStateStoreFields};

/// Workspace Tab Bar component
#[component]
pub fn WorkspaceTabBar(
    workspaces: Memo<Vec<Workspace>>,
    current_workspace: ReadSignal<u32>,
    set_current_workspace: WriteSignal<u32>,
) -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    let store = use_app_store();
    let (adding, set_adding) = signal(false);
    let (new_name, set_new_name) = signal(String::new());
    
    let on_add = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name = new_name.get();
        if name.is_empty() { return; }
        
        spawn_local(async move {
            if let Ok(new_ws) = commands::create_workspace(&name).await {
                // Fine-grained update: push new workspace to store
                store.workspaces().write().push(new_ws);
            }
        });
        
        set_new_name.set(String::new());
        set_adding.set(false);
    };
    
    view! {
        <div class="workspace-tab-bar">
            <For
                each=move || workspaces.get()
                key=|ws| ws.id
                children=move |ws| {
                    let id = ws.id;
                    let is_active = move || current_workspace.get() == id;
                    let tab_class = move || {
                        if is_active() { "workspace-tab active" } else { "workspace-tab" }
                    };
                    
                    view! {
                        <button
                            class=tab_class
                            on:click=move |_| set_current_workspace.set(id)
                        >
                            {ws.name.clone()}
                        </button>
                    }
                }
            />
            
            {move || if adding.get() {
                view! {
                    <form class="workspace-add-form" on:submit=on_add>
                        <input
                            type="text"
                            placeholder="Workspace name"
                            prop:value=move || new_name.get()
                            on:input=move |ev| set_new_name.set(event_target_value(&ev))
                        />
                        <button type="submit">"+"</button>
                        <button type="button" on:click=move |_| set_adding.set(false)>"×"</button>
                    </form>
                }.into_any()
            } else {
                view! {
                    <button
                        class="workspace-add-btn"
                        on:click=move |_| set_adding.set(true)
                    >
                        "+"
                    </button>
                }.into_any()
            }}
        </div>
    }
}
