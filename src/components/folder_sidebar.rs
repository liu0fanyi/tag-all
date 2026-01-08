use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::commands;
use crate::models::WorkspaceDir;

#[component]
pub fn FolderSidebar(
    workspace_id: u32,
    selected_path: Signal<Option<String>>,
    set_selected_path: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (dirs, set_dirs) = signal(Vec::<WorkspaceDir>::new());

    // Load dirs
    let load_dirs = move || {
        spawn_local(async move {
            if let Ok(loaded) = commands::list_workspace_paths(workspace_id).await {
                set_dirs.set(loaded);
            }
        });
    };

    Effect::new(move |_| {
        load_dirs();
    });

    let add_folder = move |_| {
        spawn_local(async move {
            if let Ok(Some(path)) = commands::pick_folder().await {
                // Add to workspace
                if let Ok(_) = commands::add_workspace_path(workspace_id, &path).await {
                    load_dirs();
                }
            }
        });
    };
    
    let remove_folder = move |id: u32, e: web_sys::MouseEvent| {
        e.stop_propagation();
        spawn_local(async move {
            if let Ok(_) = commands::remove_workspace_path(id).await {
                 load_dirs();
                 // Deselect if removed
                 // ... logic needed but skip for now
                 set_selected_path.set(None);
            }
        });
    };

    view! {
        <div class="folder-sidebar">
            <div class="sidebar-header">
                <h3>"Folders"</h3>
                <button class="add-folder-btn" on:click=add_folder>"+"</button>
            </div>
            
            <ul class="folder-list">
                <For
                    each=move || dirs.get()
                    key=|dir| dir.id
                    children=move |dir| {
                        let path = dir.path.clone();
                        let path_clone = path.clone();
                        let is_selected = move || selected_path.get() == Some(path_clone.clone());
                        let id = dir.id;
                        
                        view! {
                            <li 
                                class=move || if is_selected() { "folder-item active" } else { "folder-item" }
                                on:click=move |_| {
                                    web_sys::console::log_1(&format!("[FolderSidebar] Clicked: {}", path.clone()).into());
                                    set_selected_path.set(Some(path.clone()));
                                }
                            >
                                <span class="folder-icon">"📁"</span>
                                <span class="folder-name">{dir.path}</span>
                                <button class="delete-folder-btn" on:click=move |e| remove_folder(id, e)>"×"</button>
                            </li>
                        }
                    }
                />
            </ul>
        </div>
    }
}
