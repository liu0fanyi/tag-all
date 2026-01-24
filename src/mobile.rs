use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::commands;
use crate::models::{Item, Tag};
use std::collections::{HashSet, HashMap};
use crate::commands::CreateItemArgs;
use crate::tree::flatten_tree;
use tauri_sync_db_frontend::{GenericBottomNav, SyncSettingsForm};

/// Mobile view selection
#[derive(Clone, Copy, PartialEq)]
enum MobileView {
    Main,
    Settings,
    Editor,
}

#[component]
pub fn MobileApp() -> impl IntoView {
    let (current_view, set_current_view) = signal(MobileView::Main);
    let (items, set_items) = signal(Vec::<Item>::new());
    let (new_todo, set_new_todo) = signal(String::new());
    
    // Editor state
    let (editing_item_id, set_editing_item_id) = signal::<Option<u32>>(None);
    let (edit_title, set_edit_title) = signal(String::new());
    let (edit_memo, set_edit_memo) = signal(String::new());

    // Tag Sidebar State
    let (sidebar_open, set_sidebar_open) = signal(false);
    let (all_tags, set_all_tags) = signal(Vec::<Tag>::new());
    let (filter_tags, set_filter_tags) = signal(HashSet::<String>::new());
    let (filter_op_and, set_filter_op_and) = signal(false); // Default OR
    
    // Cache for item tags: ItemID -> TagNames
    let (item_tags_cache, set_item_tags_cache) = signal(HashMap::<u32, Vec<String>>::new());

    // Load items helper

    // Load items helper
    let load_items = move |set_items: WriteSignal<Vec<Item>>| {
        spawn_local(async move {
            if let Ok(loaded) = commands::list_items_by_workspace(1).await {
                set_items.set(loaded);
            }
        });
    };

    // Load ALL tags for flattening
    let load_tags = move |set_all_tags: WriteSignal<Vec<Tag>>| {
        spawn_local(async move {
            if let Ok(tags) = commands::list_tags().await {
                set_all_tags.set(tags);
            }
        });
    };
    
    // Tag Tree Helper: (id, name, children_ids)
    // Actually, getting children from flat list requires knowing parent_id. 
    // Tag struct has NO parent_id field. 
    // So we MUST use commands to get structure provided by backend (db relations).
    // The backend `list_tags` returns just tags. 
    // We need `get_root_tags` and `get_tag_children`.
    
    let (root_tags, set_root_tags) = signal(Vec::<Tag>::new());
    
    let load_root_tags = move |set_root_tags: WriteSignal<Vec<Tag>>| {
        spawn_local(async move {
             if let Ok(roots) = commands::get_root_tags().await {
                 set_root_tags.set(roots);
             }
        });
    };

    // Initial load
    Effect::new(move |_| {
        load_items(set_items);
        load_tags(set_all_tags); // Keep this for now if used elsewhere? 
        // actually filter depends on all_tags, but we want tree in sidebar.
        load_root_tags(set_root_tags);
    });

    // Fetch tags for items when items are loaded
    Effect::new(move |_| {
        let current_items = items.get();
        for item in current_items {
            let id = item.id;
            // Only fetch if not in cache (optimization)
            if !item_tags_cache.with(|c| c.contains_key(&id)) {
                spawn_local(async move {
                    if let Ok(tags) = commands::get_item_tags(id).await {
                         let tag_names: Vec<String> = tags.into_iter().map(|t| t.name).collect();
                         set_item_tags_cache.update(|c| {
                             c.insert(id, tag_names);
                         });
                    }
                });
            }
        }
    });

    let add_todo = move |_| {
        let content = new_todo.get();
        if content.is_empty() { return; }
        
        spawn_local(async move {
            let args = commands::CreateItemArgs {
                workspace_id: Some(1),
                text: &content,
                item_type: Some("note"),
                parent_id: None,
            };

            if let Ok(_) = commands::create_item(&args).await {
                set_new_todo.set(String::new());
                if let Ok(loaded) = commands::list_items_by_workspace(1).await {
                    set_items.set(loaded);
                }
            }
        });
    };

    let toggle_item = move |id: u32| {
        spawn_local(async move {
            let _ = commands::toggle_item(id).await;
            if let Ok(loaded) = commands::list_items_by_workspace(1).await {
                set_items.set(loaded);
            }
        });
    };

    let toggle_collapse = move |id: u32| {
        spawn_local(async move {
            let _ = commands::toggle_collapsed(id).await;
            if let Ok(loaded) = commands::list_items_by_workspace(1).await {
                 set_items.set(loaded);
            }
        });
    };
    
    // Open editor
    let open_editor = move |item: Item| {
        set_editing_item_id.set(Some(item.id));
        set_edit_title.set(item.text);
        set_edit_memo.set(item.memo.unwrap_or_default());
        set_current_view.set(MobileView::Editor);
    };
    
    // Save editor content
    // Save editor content
    let save_editor = move |_| {
        if let Some(id) = editing_item_id.get() {
            let title = edit_title.get();
            let memo = edit_memo.get();
             spawn_local(async move {
                // Update title and memo
                let _ = commands::update_item_full(
                    id, 
                    Some(&title), 
                    None, 
                    None, 
                    Some(&memo)
                ).await;
                
                // Reload list
                if let Ok(loaded) = commands::list_items_by_workspace(1).await {
                    set_items.set(loaded);
                }
                set_current_view.set(MobileView::Main);
            });
        }
    };

    let toggle_filter_tag = move |tag_name: String| {
        set_filter_tags.update(|s| {
            if s.contains(&tag_name) {
                s.remove(&tag_name);
            } else {
                s.insert(tag_name);
            }
        });
    };

    let filtered_items = move || {
        let all = items.get();
        let selected = filter_tags.get();
        
        // If no filter, return flattened tree
        if selected.is_empty() {
             return flatten_tree(&all);
        }
        
        let is_and = filter_op_and.get();
        let cache = item_tags_cache.get();

        // If filtered, return flat list with depth 0
        all.into_iter().filter(|item| {
             if let Some(tags) = cache.get(&item.id) {
                 if is_and {
                     selected.iter().all(|t| tags.contains(t))
                 } else {
                     selected.iter().any(|t| tags.contains(t))
                 }
             } else {
                 false 
             }
        })
        .map(|item| (item, 0)) // Depth 0 for filtered results
        .collect::<Vec<(Item, usize)>>()
    };

    view! {
        <div class="mobile-app-container" style="display: flex; flex-direction: column; height: 100vh;">
            // Main content area
            <div style="flex: 1; overflow-y: auto; padding-bottom: 70px;">
                {move || match current_view.get() {
                    MobileView::Main => view! {
                        <div style="padding: 20px; font-family: sans-serif;">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px; padding-top: env(safe-area-inset-top);">
                                <h1 style="margin: 0;">"Todos & Tags"</h1>
                                <button 
                                    on:click=move |_| set_sidebar_open.set(true)
                                    style="padding: 5px 10px; border: 1px solid #ccc; border-radius: 4px; background: white;"
                                >
                                    "🏷️"
                                </button>
                            </div>
                            
                            <div class="add-form" style="display: flex; gap: 10px; margin-bottom: 20px;">
                                <input
                                    type="text"
                                    prop:value=new_todo
                                    on:input=move |ev| set_new_todo.set(event_target_value(&ev))
                                    placeholder="New Todo..."
                                    style="flex: 1; padding: 10px; border: 1px solid #ccc; border-radius: 4px;"
                                />
                                <button
                                    on:click=add_todo
                                    style="padding: 10px 20px; background: #007bff; color: white; border: none; border-radius: 4px;"
                                >
                                    "Add"
                                </button>
                            </div>

                            <div class="todo-list">
                                <For
                                    each=filtered_items
                                    key=|(item, _)| item.id
                                    children=move |(item, depth)| {
                                        let item_clone = item.clone();
                                        let indent = depth * 20;
                                        
                                        // Check if item has children (for showing toggle)
                                        // Since we are flattening, we can't easily peek ahead in the iterator in this scope
                                        // But we can check if any OTHER item has this item as parent
                                        // However, iterating all items here is expensive.
                                        // Best way: compute has_children map during filtering/flattening?
                                        // Or just check if next item in filtered list has depth > current depth? 
                                        // Flatten_tree output puts children immediately after parent.
                                        
                                        // Optimization: Pre-calculate parent set
                                        let has_children = items.with(|list| list.iter().any(|i| i.parent_id == Some(item.id)));

                                        view! {
                                            <div 
                                                class="todo-item" 
                                                style=format!("display: flex; align-items: center; padding: 15px 10px 15px {}px; border-bottom: 1px solid #eee; cursor: pointer; user-select: none; -webkit-user-select: none;", 10 + indent)
                                                on:click=move |_| {
                                                    web_sys::console::log_1(&format!("Row clicked: {}", item_clone.id).into());
                                                    open_editor(item_clone.clone());
                                                }
                                            >
                                                // Collapse toggle
                                                <div 
                                                    style="width: 24px; height: 24px; display: flex; align-items: center; justify-content: center; margin-right: 5px;"
                                                    on:click=move |ev| {
                                                        ev.stop_propagation();
                                                        if has_children {
                                                            toggle_collapse(item.id);
                                                        }
                                                    }
                                                >
                                                    {if has_children {
                                                        if item.collapsed { "▶" } else { "▼" }
                                                    } else {
                                                        ""
                                                    }}
                                                </div>

                                                <input
                                                    type="checkbox"
                                                    checked=item.completed
                                                    on:change=move |_| toggle_item(item.id)
                                                    on:click=move |ev| ev.stop_propagation()
                                                    style="margin-right: 15px; width: 25px; height: 25px;"
                                                />
                                                <span 
                                                    style=if item.completed { "text-decoration: line-through; color: #888; flex: 1; font-size: 16px;" } else { "flex: 1; font-size: 16px;" }
                                                    on:click=move |ev| {
                                                        web_sys::console::log_1(&format!("Span clicked: {}", item_clone.id).into());
                                                    }
                                                >
                                                    {item.text}
                                                </span>
                                            </div>
                                        }
                                    }
                                />
                            </div>

                            // Sidebar Overlay
                            {move || if sidebar_open.get() {
                                view! {
                                    <div 
                                        style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 2000; background: rgba(0,0,0,0.5);"
                                        on:click=move |_| set_sidebar_open.set(false)
                                    >
                                        <div 
                                            style="position: absolute; right: 0; top: 0; width: 80%; height: 100%; background: white; box-shadow: -2px 0 5px rgba(0,0,0,0.2); padding: 20px; display: flex; flex-direction: column;"
                                            on:click=move |ev| ev.stop_propagation()
                                        >
                                            <div style="flex: 0 0 auto; display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                                                <h3 style="margin: 0;">"Filter Tags"</h3>
                                                <button 
                                                    on:click=move |_| set_sidebar_open.set(false)
                                                    style="border: none; background: transparent; font-size: 20px;"
                                                >
                                                    "✕"
                                                </button>
                                            </div>
                                            
                                            <div style="margin-bottom: 15px;">
                                                <label style="display: flex; align-items: center;">
                                                    <input 
                                                        type="checkbox" 
                                                        prop:checked=filter_op_and
                                                        on:change=move |ev| set_filter_op_and.set(event_target_checked(&ev))
                                                        style="margin-right: 10px;" 
                                                    />
                                                    "Match All (AND)"
                                                </label>
                                            </div>

                                            <div style="flex: 1; overflow-y: auto; padding-bottom: 50px;">
                                                // Recursive Tag Tree Rendering
                                                <For
                                                    each=move || root_tags.get()
                                                    key=|tag| tag.id
                                                    children=move |tag| {
                                                        view! {
                                                            <MobileTagNode 
                                                                tag=tag 
                                                                depth=0 
                                                                filter_tags=filter_tags.into() 
                                                                set_filter_tags=set_filter_tags
                                                            />
                                                        }
                                                    }
                                                />
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span style="display: none;"></span> }.into_any()
                            }}
                        </div>
                    }.into_any(),
                    MobileView::Settings => view! {
                        <SyncSettingsForm on_back=move || set_current_view.set(MobileView::Main) />
                    }.into_any(),
                     MobileView::Editor => view! {
                         <div style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 1000; background: white; display: flex; flex-direction: column;">
                            <div style="flex: 0 0 auto; padding: 20px; border-bottom: 1px solid #eee; display: flex; justify-content: space-between; align-items: center; background: white;">
                                <button 
                                    on:click=move |_| set_current_view.set(MobileView::Main)
                                    style="padding: 8px 15px; background: #eee; border: none; border-radius: 4px;"
                                >
                                    "Cancel"
                                </button>
                                <h3 style="margin: 0;">"Edit Todo"</h3>
                                <button 
                                    on:click=save_editor
                                    style="padding: 8px 15px; background: #007bff; color: white; border: none; border-radius: 4px;"
                                >
                                    "Save"
                                </button>
                            </div>
                            
                            <div style="flex: 1; display: flex; flex-direction: column; padding: 20px; overflow: hidden;">
                                <div style="margin-bottom: 15px; flex-shrink: 0;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Title"</label>
                                    <input
                                        type="text"
                                        prop:value=edit_title
                                        on:input=move |ev| set_edit_title.set(event_target_value(&ev))
                                        style="width: 100%; padding: 10px; border: 1px solid #ccc; border-radius: 4px; font-size: 16px; box-sizing: border-box;"
                                    />
                                </div>
                                
                                <div style="flex: 1; display: flex; flex-direction: column; min-height: 0;">
                                    <label style="display: block; margin-bottom: 5px; font-weight: bold;">"Memo (Markdown)"</label>
                                    <textarea
                                        prop:value=edit_memo
                                        on:input=move |ev| set_edit_memo.set(event_target_value(&ev))
                                        style="flex: 1; width: 100%; padding: 10px; border: 1px solid #ccc; border-radius: 4px; font-family: monospace; resize: none; box-sizing: border-box;"
                                        placeholder="# Write markdown here..."
                                    ></textarea>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                }}
            </div>
            
            // Bottom navigation (Hide in Editor mode)
            {move || if current_view.get() != MobileView::Editor {
                view! {
                    <GenericBottomNav on_settings_click=Box::new(move || {
                        set_current_view.update(|v| {
                            *v = if *v == MobileView::Settings { 
                                MobileView::Main 
                            } else { 
                                MobileView::Settings 
                            };
                        });
                    })>
                        <button
                            class=move || if current_view.get() == MobileView::Main { "mobile-nav-item active" } else { "mobile-nav-item" }
                            on:click=move |_| set_current_view.set(MobileView::Main)
                        >
                            <div class="mobile-nav-icon">"📝"</div>
                            <div class="mobile-nav-label">"待办"</div>
                        </button>
                    </GenericBottomNav>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}

/// Recursive Mobile Tag Node
#[component]
fn MobileTagNode(
    tag: Tag,
    depth: usize,
    filter_tags: Signal<HashSet<String>>,
    set_filter_tags: WriteSignal<HashSet<String>>,
) -> impl IntoView {
    let id = tag.id;
    let name = tag.name.clone();
    let name_for_select = name.clone();
    let name_for_toggle_div = name.clone();
    let name_for_toggle_input = name.clone();
    
    // Load children
    let (children, set_children) = signal(Vec::<Tag>::new());
    let (expanded, set_expanded) = signal(true); // Default expanded for visibility

    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(child_tags) = commands::get_tag_children(id).await {
                set_children.set(child_tags);
            }
        });
    });

    let is_selected = move || filter_tags.with(|s| s.contains(&name_for_select));
    let has_children = move || !children.get().is_empty();
    
    // Toggle filter logic used by parent
    let toggle_filter = move |tag_name: String| {
        set_filter_tags.update(|s| {
            if s.contains(&tag_name) {
                s.remove(&tag_name);
            } else {
                s.insert(tag_name);
            }
        });
    };

    view! {
        <div style="display: flex; flex-direction: column;">
            <div 
                style=format!("padding: 10px 10px 10px {}px; border-bottom: 1px solid #eee; display: flex; align-items: center;", 10 + depth * 20)
                on:click=move |_| toggle_filter(name_for_toggle_div.clone())
            >
                // Expand toggle (only if children)
                 <div 
                    style="width: 24px; height: 24px; display: flex; align-items: center; justify-content: center; margin-right: 5px;"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        if has_children() {
                            set_expanded.update(|v| *v = !*v);
                        }
                    }
                >
                    {move || if has_children() {
                        if expanded.get() { "▼" } else { "▶" }
                    } else {
                        "·" // Placeholder
                    }}
                </div>

                <input 
                    type="checkbox" 
                    prop:checked=is_selected
                    style="margin-right: 10px;"
                    on:click=move |ev| ev.stop_propagation()
                    on:change=move |_| toggle_filter(name_for_toggle_input.clone())
                />
                <span>{tag.name}</span>
            </div>
            
            // Children
            {move || if expanded.get() {
                view! {
                    <div>
                        <For
                            each=move || children.get()
                            key=|child| child.id
                            children=move |child| {
                                view! {
                                    <MobileTagNode 
                                        tag=child 
                                        depth=depth + 1 
                                        filter_tags=filter_tags 
                                        set_filter_tags=set_filter_tags
                                    />
                                }
                            }
                        />
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}

