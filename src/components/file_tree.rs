use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::commands;
use crate::models::{WorkspaceDir, FileViewItem, Tag};
use crate::components::{TagDndContext, EditTarget};
use leptos_dragdrop::{make_on_mouseleave, make_on_file_mouseenter, DropTarget};

#[component]
pub fn FileTree(
    workspace_id: u32,
    set_selected_file: WriteSignal<Option<FileViewItem>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let (dirs, set_dirs) = signal(Vec::<WorkspaceDir>::new());
    
    // Load workspace directories
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
                if let Ok(_) = commands::add_workspace_path(workspace_id, &path).await {
                    load_dirs();
                }
            }
        });
    };

    view! {
        <div class="file-tree-container">
            <div class="tree-header">
                <h3>"Folders"</h3>
                <button class="add-folder-btn" on:click=add_folder title="Add Folder">"+"</button>
            </div>
            <div class="tree-content">
                <For
                    each=move || dirs.get()
                    key=|dir| dir.id
                    children=move |dir| {
                        view! {
                            <FileTreeRow 
                                dir=dir.clone() 
                                on_change=move || load_dirs() 
                                set_selected_file=set_selected_file
                                set_editing_target=set_editing_target
                            />
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
fn FileTreeRow(
    dir: WorkspaceDir,
    #[prop(into)]
    on_change: Callback<()>,
    set_selected_file: WriteSignal<Option<FileViewItem>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let (collapsed, set_collapsed) = signal(dir.collapsed);
    let (files, set_files) = signal(Vec::<FileViewItem>::new());
    let (loading, set_loading) = signal(false);
    let (loaded_once, set_loaded_once) = signal(false);
    
    let tag_dnd = use_context::<TagDndContext>().expect("TagDndContext should be provided");
    let dnd = tag_dnd.dnd;

    // Load files logic
    let dir_path = dir.path.clone();
    let load_files = move || {
        let path = dir_path.clone();
        set_loading.set(true);
        
        spawn_local(async move {
             match commands::list_directory(&path).await {
                Ok(res) => {
                    set_files.set(res);
                },
                Err(_) => {
                    // Error handling
                }
            }
            set_loading.set(false);
            set_loaded_once.set(true);
        });
    };

    // Load files when expanding if not loaded
    let load_files_cancel = load_files.clone();
    let toggle_collapse = move |e: web_sys::MouseEvent| {
        e.stop_propagation(); // Prevent tree ripple?
        let new_state = !collapsed.get();
        set_collapsed.set(new_state);
        
        spawn_local(async move {
            let _ = commands::toggle_workspace_dir_collapsed(dir.id, new_state).await;
        });
        
        if !new_state && !loaded_once.get() {
            load_files_cancel();
        }
    };
    
    // Check if initial state is expanded, load immediately
    let load_files_init = load_files.clone();
    Effect::new(move |_| {
        if !dir.collapsed && !loaded_once.get() {
            load_files_init();
        }
    });
    
    // Delete folder
    let delete_folder = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        // TODO: Confirm dialog?
        spawn_local(async move {
             if let Ok(_) = commands::remove_workspace_path(dir.id).await {
                 on_change.run(());
             }
        });
    };
    
    // Refresh files manually
    let load_files_refresh = load_files.clone();
    let refresh_files = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        if !collapsed.get() {
            load_files_refresh();
        } else {
            // If collapsed, expand and load
            set_collapsed.set(false);
            load_files_refresh();
             spawn_local(async move {
                let _ = commands::toggle_workspace_dir_collapsed(dir.id, false).await;
            });
        }
    };

    // Global reload trigger listener
    let ctx = use_context::<crate::context::AppContext>().expect("AppContext");
    let reload_trigger = ctx.reload_trigger;
    let load_files_reload = load_files.clone();
    Effect::new(move |_| {
        let _ = reload_trigger.get();
        // Use untracked to avoid reacting to these changes, only reload_trigger
        if !collapsed.get_untracked() && loaded_once.get_untracked() {
             load_files_reload();
        }
    });
    
    // File Context Menu Handler
    let on_file_context_menu = move |ev: web_sys::MouseEvent, file: FileViewItem| {
        ev.prevent_default();
        ev.stop_propagation();
        
        if let Some(item) = file.db_item {
            set_editing_target.set(Some(EditTarget::Item(item.id, item.text)));
        } else {
            let path = file.path.clone();
            // Use file name as initial text for new item
            let name = file.name.clone(); 
            spawn_local(async move {
                 if let Ok(item) = commands::ensure_file_item(&path).await {
                     set_editing_target.set(Some(EditTarget::Item(item.id, item.text)));
                     // Reload files to reflect that this file now has a DB item? 
                     // Or just let UI update via reload trigger if ensure_file_item doesn't signal
                     // But TagEditor edits require an Item ID.
                 }
            });
        }
        
        spawn_local(async {
            let _ = commands::resize_window(1100, 700).await;
        });
    };

    view! {
        <div class="tree-row-container">
            // Folder Header
            <div class="tree-folder-header" on:click=toggle_collapse>
                <div class="folder-info">
                    <span class="toggle-icon">
                        {move || if collapsed.get() { "▶" } else { "▼" }}
                    </span>
                    <span class="folder-icon">"📁"</span>
                    <span class="folder-name" title=dir.path.clone()>{dir.path.clone()}</span>
                </div>
                <div class="folder-actions">
                     <button class="action-btn" on:click=refresh_files title="Refresh">"↻"</button>
                     <button class="action-btn delete" on:click=delete_folder title="Remove">"×"</button>
                </div>
            </div>
            
            // Files List (Children)
            <Show when=move || !collapsed.get()>
                <div class="tree-children">
                     <Show when=move || !loading.get() fallback=|| view! { <div class="loading small">"Loading..."</div> }>
                        <For
                            each=move || files.get()
                            key=|f| f.path.clone()
                            children=move |file| {
                                let file_path = file.path.clone();
                                let file_path_for_drop = file_path.clone();
                                let file_path_for_click = file_path.clone();
                                
                                let is_tagged = file.db_item.is_some();
                                let tags_empty = file.tags.is_empty();
                                let file_tags = file.tags.clone();
                                
                                let item_clone = file.clone();
                                let item_clone_for_menu = file.clone();
                                
                                // DnD
                                let on_mouseenter = make_on_file_mouseenter(dnd, file_path.clone());
                                let on_mouseleave = make_on_mouseleave(dnd);
                                let is_drop_target = move || {
                                    matches!(dnd.drop_target_read.get(), Some(DropTarget::File(ref p)) if *p == file_path_for_drop)
                                };

                                view! {
                                    <div 
                                        class="tree-file-item"
                                        class:tagged=is_tagged
                                        class:drop-target=is_drop_target
                                        on:click=move |_| {
                                            set_selected_file.set(Some(item_clone.clone()));
                                            // Open file externally
                                            let p = file_path_for_click.clone();
                                            spawn_local(async move {
                                                let _ = commands::open_file(&p).await;
                                            });
                                        }
                                        on:contextmenu=move |ev| on_file_context_menu(ev, item_clone_for_menu.clone())
                                        on:mouseenter=on_mouseenter
                                        on:mouseleave=on_mouseleave
                                    >
                                        <span class="file-icon">{if file.is_dir { "📂" } else { "📄" }}</span>
                                        <span class="file-name">{file.name}</span>
                                        
                                        // Display Tags
                                        <div class="file-tags">
                                            <For
                                                each=move || file_tags.clone()
                                                key=|t| t.id
                                                children=move |tag| {
                                                    view! {
                                                        <span 
                                                            class="file-tag-chip"
                                                            style=format!("background-color: {}", tag.color.unwrap_or_else(|| "#eee".into()))
                                                        >
                                                            {tag.name}
                                                        </span>
                                                    }
                                                }
                                            />
                                        </div>
                                        
                                        {if is_tagged && tags_empty {
                                            view! { <span></span> }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                }
                            }
                        />
                         <Show when=move || files.get().is_empty()>
                            <div class="empty-folder">"Empty"</div>
                        </Show>
                     </Show>
                </div>
            </Show>
        </div>
    }
}
