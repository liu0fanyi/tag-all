//! Workspace Tab Bar Component
//!
//! Tab bar for switching between workspaces in the main content area.

use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::models::Workspace;
use crate::commands;
use crate::store::{use_app_store, AppStateStoreFields};

/// Fixed workspace IDs (1-4) cannot be edited or deleted
const FIXED_WORKSPACE_MAX_ID: u32 = 4;

/// Workspace Tab Bar component
#[component]
pub fn WorkspaceTabBar(
    workspaces: Memo<Vec<Workspace>>,
    current_workspace: ReadSignal<u32>,
    set_current_workspace: WriteSignal<u32>,
) -> impl IntoView {
    let store = use_app_store();
    let (adding, set_adding) = signal(false);
    let (new_name, set_new_name) = signal(String::new());
    // Track which workspace is being edited (None = not editing)
    let (editing_id, set_editing_id) = signal::<Option<u32>>(None);
    let (edit_name, set_edit_name) = signal(String::new());
    
    let on_add = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name = new_name.get();
        if name.is_empty() { return; }
        
        spawn_local(async move {
            if let Ok(new_ws) = commands::create_workspace(&name).await {
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
                    let is_fixed = id <= FIXED_WORKSPACE_MAX_ID;
                    let is_active = move || current_workspace.get() == id;
                    let is_editing = move || editing_id.get() == Some(id);
                    let tab_class = move || {
                        if is_active() { "workspace-tab active" } else { "workspace-tab" }
                    };
                    
                    // Reactively get the workspace name from store
                    let get_ws_name = move || {
                        store.workspaces().get()
                            .iter()
                            .find(|w| w.id == id)
                            .map(|w| w.name.clone())
                            .unwrap_or_default()
                    };
                    
                    view! {
                        {move || {
                            if is_editing() {
                                // Edit mode: show input field and delete button
                                view! {
                                    <form class="workspace-edit-form" on:submit=move |ev| {
                                        ev.prevent_default();
                                        let new_name_val = edit_name.get();
                                        if new_name_val.is_empty() {
                                            set_editing_id.set(None);
                                            return;
                                        }
                                        let id_to_rename = id;
                                        spawn_local(async move {
                                            if commands::rename_workspace(id_to_rename, &new_name_val).await.is_ok() {
                                                // Update local store
                                                store.workspaces().update(|workspaces| {
                                                    if let Some(ws) = workspaces.iter_mut().find(|w| w.id == id_to_rename) {
                                                        ws.name = new_name_val;
                                                    }
                                                });
                                            }
                                        });
                                        set_editing_id.set(None);
                                    }>
                                        <input
                                            type="text"
                                            class="workspace-edit-input"
                                            prop:value=move || edit_name.get()
                                            on:input=move |ev| set_edit_name.set(event_target_value(&ev))
                                            on:blur=move |_| set_editing_id.set(None)
                                        />
                                        <button type="button" class="workspace-delete-btn" on:mousedown=move |_| {
                                            let id_to_delete = id;
                                            spawn_local(async move {
                                                if commands::delete_workspace(id_to_delete).await.is_ok() {
                                                    store.workspaces().write().retain(|w| w.id != id_to_delete);
                                                    if current_workspace.get_untracked() == id_to_delete {
                                                        set_current_workspace.set(1);
                                                    }
                                                }
                                            });
                                            set_editing_id.set(None);
                                        }>
                                            "×"
                                        </button>
                                    </form>
                                }.into_any()
                            } else {
                                // Normal mode: show tab button
                                let current_name = get_ws_name();
                                view! {
                                    <button
                                        class=tab_class
                                        on:click=move |_| set_current_workspace.set(id)
                                        on:dblclick=move |_| {
                                            if !is_fixed {
                                                set_edit_name.set(get_ws_name());
                                                set_editing_id.set(Some(id));
                                            }
                                        }
                                    >
                                        {current_name}
                                    </button>
                                }.into_any()
                            }
                        }}
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

