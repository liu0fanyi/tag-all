//! Tag-All Frontend App
//!
//! Main application component with multi-column layout.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::models::{Item, Tag, Workspace};
use crate::commands;
use crate::context::AppContext;
use crate::components::{NewItemForm, TagColumn, TagEditor, ItemTreeView, EditTarget, WorkspaceTabBar, MemoEditorColumn, TitleBar};

/// Filter mode for tag-based item filtering
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FilterMode {
    And, // Item must have ALL selected tags
    Or,  // Item must have ANY of selected tags
}

/// Sort mode for item display (temporary, not persisted)
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum SortMode {
    #[default]
    Position,  // Default: sort by position
    NameFirst, // Uncompleted first, then by name
    TagFirst,  // Sort by first tag name
}

#[component]
pub fn App() -> impl IntoView {
    // State
    let (items, set_items) = signal(Vec::<Item>::new());
    let (tags, set_tags) = signal(Vec::<Tag>::new());
    let (workspaces, set_workspaces) = signal(Vec::<Workspace>::new());
    let (current_workspace, set_current_workspace) = signal(1u32);
    let (adding_under, set_adding_under) = signal::<Option<u32>>(None);
    let (reload_trigger, set_reload_trigger) = signal(0u32);
    let (selected_item, set_selected_item) = signal::<Option<u32>>(None);
    
    // Tag filtering: multi-select support
    let (selected_tags, set_selected_tags) = signal(Vec::<u32>::new());
    let (filter_mode, set_filter_mode) = signal(FilterMode::Or);
    
    // Temporary sort mode (not persisted)
    let (sort_mode, set_sort_mode) = signal(SortMode::Position);
    
    // Right-click opens properties editor
    let (editing_target, set_editing_target) = signal::<Option<EditTarget>>(None);
    // Right-click on Item opens memo editor
    let (memo_editing_target, set_memo_editing_target) = signal::<Option<EditTarget>>(None);
    
    // Pin state (always on top)
    let (is_pinned, set_is_pinned) = signal(false);
    
    // Load initial pinned state
    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(Some(state)) = commands::load_window_state().await {
                set_is_pinned.set(state.pinned);
                if state.pinned {
                    let _ = commands::set_pinned(true).await;
                }
            }
        });
    });

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
        spawn_local(async move {
            if let Ok(loaded) = commands::list_items_by_workspace(ws_id).await {
                set_items.set(loaded);
            }
            if let Ok(loaded) = commands::list_tags().await {
                set_tags.set(loaded);
            }
        });
    });
    
    // Toggle filter mode
    let toggle_filter_mode = move |_| {
        set_filter_mode.update(|m| {
            *m = match m {
                FilterMode::And => FilterMode::Or,
                FilterMode::Or => FilterMode::And,
            };
        });
    };
    
    // Clear tag filter
    let clear_filter = move |_| {
        set_selected_tags.set(Vec::new());
    };

    view! {
        <div class="app-container">
            // Custom Title Bar
            <TitleBar is_pinned=is_pinned set_is_pinned=set_is_pinned />
            
            <div class="app-layout">
                // Left: Tag Column
                <TagColumn
                    selected_tags=selected_tags
                    set_selected_tags=set_selected_tags
                    editing_target=editing_target
                    set_editing_target=set_editing_target
                    set_memo_editing_target=set_memo_editing_target
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
                
                // Filter mode toggle (shown when tags are selected)
                <Show when=move || !selected_tags.get().is_empty()>
                    <div class="filter-bar">
                        <span class="filter-label">"筛选:"</span>
                        <button 
                            class=move || if filter_mode.get() == FilterMode::And { "filter-btn active" } else { "filter-btn" }
                            on:click=toggle_filter_mode
                        >
                            {move || if filter_mode.get() == FilterMode::And { "AND" } else { "OR" }}
                        </button>
                        <button class="filter-clear-btn" on:click=clear_filter>"清除"</button>
                    </div>
                </Show>
                
                <NewItemForm />
                
                // Sort toggle buttons
                <div class="sort-bar">
                    <button
                        class=move || if sort_mode.get() == SortMode::NameFirst { "sort-btn active" } else { "sort-btn" }
                        on:click=move |_| {
                            set_sort_mode.update(|m| {
                                *m = if *m == SortMode::NameFirst { SortMode::Position } else { SortMode::NameFirst };
                            });
                        }
                    >
                        "未完成优先"
                    </button>
                    <button
                        class=move || if sort_mode.get() == SortMode::TagFirst { "sort-btn active" } else { "sort-btn" }
                        on:click=move |_| {
                            set_sort_mode.update(|m| {
                                *m = if *m == SortMode::TagFirst { SortMode::Position } else { SortMode::TagFirst };
                            });
                        }
                    >
                        "按标签排序"
                    </button>
                    <button
                        class="sort-btn reset"
                        title="重置所有已完成的任务"
                        on:click=move |_| {
                            let ws = current_workspace.get();
                            spawn_local(async move {
                                let _ = commands::reset_all_items(ws).await;
                            });
                            set_reload_trigger.update(|n| *n += 1);
                        }
                    >
                        "🔄 重置"
                    </button>
                </div>
                
                <ItemTreeView
                    items=items
                    selected_item=selected_item
                    set_selected_item=set_selected_item
                    selected_tags=selected_tags
                    filter_mode=filter_mode
                    sort_mode=sort_mode
                    editing_target=editing_target
                    set_editing_target=set_editing_target
                    memo_editing_target=memo_editing_target
                    set_memo_editing_target=set_memo_editing_target
                />
                
                <p class="item-count">{move || format!("{} items, {} tags", items.get().len(), tags.get().len())}</p>
            </main>
            
            // Right: Tag Editor (shown on right-click)
            <TagEditor
                editing_target=editing_target
                set_editing_target=set_editing_target
            />
            
            // Far Right: Memo Editor (shown on Item right-click)
            <MemoEditorColumn
                editing_target=memo_editing_target
                set_editing_target=set_memo_editing_target
            />
            </div>
        </div>
    }
}
