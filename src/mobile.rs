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
}

#[component]
pub fn MobileApp() -> impl IntoView {
    let (current_view, set_current_view) = signal(MobileView::Main);
    let (items, set_items) = signal(Vec::<Item>::new());
    let (new_todo, set_new_todo) = signal(String::new());

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
                                        view! {
                                            <div class="todo-item" style="display: flex; align-items: center; padding: 10px; border-bottom: 1px solid #eee;">
                                                <input
                                                    type="checkbox"
                                                    checked=item.completed
                                                    on:change=move |_| toggle_item(item.id)
                                                    style="margin-right: 10px; width: 20px; height: 20px;"
                                                />
                                                <span style=if item.completed { "text-decoration: line-through; color: #888;" } else { "" }>
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
                }}
            </div>
            
            // Bottom navigation
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
        </div>
    }
}
