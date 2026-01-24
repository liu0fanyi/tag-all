use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::commands;
use crate::models::Item;
use crate::commands::CreateItemArgs;
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

    // Load items helper
    let load_items = move |set_items: WriteSignal<Vec<Item>>| {
        spawn_local(async move {
            if let Ok(loaded) = commands::list_items_by_workspace(1).await {
                set_items.set(loaded);
            }
        });
    };

    // Initial load
    Effect::new(move |_| {
        load_items(set_items);
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

    view! {
        <div class="mobile-app-container" style="display: flex; flex-direction: column; height: 100vh;">
            // Main content area
            <div style="flex: 1; overflow-y: auto; padding-bottom: 70px;">
                {move || match current_view.get() {
                    MobileView::Main => view! {
                        <div style="padding: 20px; font-family: sans-serif;">
                            <h1>"My Todos"</h1>
                            
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
                                    each=move || items.get()
                                    key=|item| item.id
                                    children=move |item| {
                                        let item_clone = item.clone();
                                        view! {
                                            <div 
                                                class="todo-item" 
                                                style="display: flex; align-items: center; padding: 15px 10px; border-bottom: 1px solid #eee; cursor: pointer; user-select: none; -webkit-user-select: none;"
                                                on:click=move |_| {
                                                    web_sys::console::log_1(&format!("Row clicked: {}", item_clone.id).into());
                                                    open_editor(item_clone.clone());
                                                }
                                            >
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

