//! Tag-All Frontend App
//!
//! Main application component with multi-column layout.

use leptos::prelude::*;
use leptos::task::spawn_local;
use reactive_stores::Store;

use crate::models::{Item, Tag, Workspace, FileViewItem};
use crate::commands;
use crate::context::AppContext;
use crate::store::{AppState, AppStateStoreFields};
use crate::components::{NewItemForm, TagColumn, TagEditor, ItemTreeView, EditTarget, WorkspaceTabBar, MemoEditorColumn, TitleBar, SyncModal, FilesWorkspace, TagDndContext};
use crate::mobile::MobileApp;

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
    // Check mobile
    let is_mobile = move || {
        if let Some(w) = web_sys::window() {
            if let Ok(width) = w.inner_width() {
                if let Some(val) = width.as_f64() {
                    return val < 768.0;
                }
            }
        }
        false
    };

    if is_mobile() {
        return view! { <MobileApp /> }.into_any();
    }

    // Create and provide the global store
    let store = Store::new(AppState::new());
    provide_context(store);
    
    // Create and provide DnD context for Tags (used by TagColumn and FileList)
    let tag_dnd = TagDndContext::new();
    provide_context(tag_dnd);
    
    // Derived signals from store for compatibility
    let items = Memo::new(move |_| store.items().get());
    let tags = Memo::new(move |_| store.tags().get());
    let workspaces = Memo::new(move |_| store.workspaces().get());
    let (current_workspace, set_current_workspace) = signal(1u32);
    let (adding_under, set_adding_under) = signal::<Option<u32>>(None);
    let (reload_trigger, set_reload_trigger) = signal(0u32);
    let (selected_item, set_selected_item) = signal::<Option<u32>>(None);
    // Multi-select state for shift-click
    let (selected_items, set_selected_items) = signal(Vec::<u32>::new());
    
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
    
    // Files Workspace Selection
    let (selected_file, set_selected_file) = signal::<Option<FileViewItem>>(None);
    
    // Sync state
    let (sync_url, set_sync_url) = signal(String::new());
    let (sync_token, set_sync_token) = signal(String::new());
    let (sync_status, set_sync_status) = signal("idle".to_string());
    let (sync_msg, set_sync_msg) = signal(String::new());
    let (show_sync_modal, set_show_sync_modal) = signal(false);
    

    
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
    
    // Startup sync: Load config and auto-sync if configured
    Effect::new(move |_| {
        spawn_local(async move {
            // First load config
            let config = commands::get_cloud_sync_config().await;
            
            // Also load initial status
            if let Ok(status) = commands::get_sync_status().await {
                 if let Some(time) = status.last_sync_time {
                     set_sync_msg.set(format!("上次同步: {}", time));
                 }
            }

            match config {
                Ok(Some(cfg)) => {
                    set_sync_url.set(cfg.url.clone());
                    set_sync_token.set(cfg.token.clone());
                    
                    // If configured, perform initial sync
                    if !cfg.url.is_empty() && !cfg.token.is_empty() {
                        // Check if cloud sync is actually enabled (might have fallen back to local)
                        let is_enabled = commands::is_cloud_sync_enabled().await;
                        
                        if !is_enabled {
                            // Config exists but disabled (likely due to validation failure)
                            set_sync_status.set("error".to_string());
                            set_sync_msg.set("配置无效，已回退到本地模式".to_string());
                            return;
                        }

                        set_sync_status.set("syncing".to_string());
                        

                        
                        // Perform sync
                        match commands::sync_cloud_db().await {
                            Ok(_) => {
                                set_sync_status.set("success".to_string());
                                // Update status msg with time
                                if let Ok(status) = commands::get_sync_status().await {
                                     if let Some(time) = status.last_sync_time {
                                         set_sync_msg.set(format!("同步完成 ({})", time));
                                     } else {
                                         set_sync_msg.set("初始同步完成".to_string());
                                     }
                                } else {
                                    set_sync_msg.set("初始同步完成".to_string());
                                }
                            }
                            Err(e) => {
                                set_sync_status.set("error".to_string());
                                set_sync_msg.set(format!("同步失败: {}", e));
                            }
                        }
                        
                        
                        // Sync done
                    } else {
                        // No config, do nothing
                    }
                }
                _ => {
                    // No config, do nothing
                }
            }
        });
    });

    // Provide context to all children
    provide_context(AppContext::new((reload_trigger, set_reload_trigger), (adding_under, set_adding_under), current_workspace));

    // Load workspaces on mount
    Effect::new(move |_| {
        let _ = reload_trigger.get();
        
        let load_workspaces = move || {
            spawn_local(async move {
                if let Ok(loaded) = commands::list_workspaces().await {
                    *store.workspaces().write() = loaded;
                }
            });
        };
        
        // Initial load
        load_workspaces();
        
        // Listen for DB ready event
        spawn_local(async move {
            let _ = commands::listen_safe("db-initialized", move |_| {
                load_workspaces();
            }).await;
        });
    });

    // Load items when workspace or trigger changes
    Effect::new(move |_| {
        let _ = reload_trigger.get();
        let ws_id = current_workspace.get();
        
        let load_items_and_tags = move || {
            spawn_local(async move {
                if let Ok(loaded) = commands::list_items_by_workspace(ws_id).await {
                    *store.items().write() = loaded;
                }
                if let Ok(loaded) = commands::list_tags().await {
                    *store.tags().write() = loaded;
                }
                if let Ok(loaded) = commands::get_root_tags().await {
                    *store.root_tags().write() = loaded;
                }
            });
        };
        
        // Initial load
        load_items_and_tags();
        
        // Listen for DB ready event
        spawn_local(async move {
            let _ = commands::listen_safe("db-initialized", move |_| {
                load_items_and_tags();
            }).await;
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
    
    // Toggle sync modal
    let toggle_sync_modal = move |_| {
        set_show_sync_modal.update(|v| *v = !*v);
    };
    
    // Save config only (no sync)
    let save_config = move |_| {
        set_sync_status.set("saving".to_string());
        set_sync_msg.set("正在保存配置...".to_string());
        
        let url_val = sync_url.get_untracked();
        let token_val = sync_token.get_untracked();
        
        spawn_local(async move {
            match commands::save_cloud_sync_config(url_val, token_val).await {
                Ok(_) => {
                    set_sync_status.set("success".to_string());
                    set_sync_msg.set("配置已保存！".to_string());
                }
                Err(e) => {
                    set_sync_status.set("error".to_string());
                    set_sync_msg.set(format!("保存失败: {}", e));
                }
            }
        });
    };
    
    // Manual sync (right-click)
    let perform_manual_sync = move |_| {
        // Check if configured
        let has_config = !sync_url.get_untracked().is_empty() && !sync_token.get_untracked().is_empty();
        
        if !has_config {
            set_sync_status.set("error".to_string());
            set_sync_msg.set("请先配置云同步（点击图标输入URL和Token）".to_string());
            set_show_sync_modal.set(true);
            return;
        }
        
        set_sync_status.set("syncing".to_string());
        set_sync_msg.set("正在检查同步状态...".to_string());
        spawn_local(async move {
            // First check if cloud sync is enabled for this session
            let is_enabled = commands::is_cloud_sync_enabled().await;
            
            if !is_enabled {
                // App is in local mode, needs restart to enable sync
                set_sync_status.set("error".to_string());
                set_sync_msg.set("当前为本地模式。配置已保存，请重启应用以启用云同步。".to_string());
                return;
            }
            
            set_sync_msg.set("正在同步...".to_string());
            match commands::sync_cloud_db().await {
                Ok(_) => {
                    set_sync_status.set("success".to_string());
                    if let Ok(status) = commands::get_sync_status().await {
                         if let Some(time) = status.last_sync_time {
                             set_sync_msg.set(format!("同步完成 ({})", time));
                         } else {
                             set_sync_msg.set("同步完成！".to_string());
                         }
                    } else {
                        set_sync_msg.set("同步完成！".to_string());
                    }
                    set_reload_trigger.update(|n| *n += 1);
                }
                Err(e) => {
                    set_sync_status.set("error".to_string());
                    set_sync_msg.set(format!("同步失败: {}", e));
                }
            }
        });
    };

    view! {
        <div class="app-container">
            // Custom Title Bar
            <TitleBar 
                is_pinned=is_pinned 
                set_is_pinned=set_is_pinned
                sync_url=sync_url.into()
                sync_token=sync_token.into()
                sync_status=sync_status.into()
                sync_msg=sync_msg.into()
                on_sync_click=Callback::new(toggle_sync_modal)
                on_sync_right_click=Callback::new(perform_manual_sync)
            />
            
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
                    

                
                // Filter mode toggle (shown when tags are selected)
                <Show when=move || current_workspace.get() == 2 fallback=move || view! {
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
                        selected_items=selected_items
                        set_selected_items=set_selected_items
                        selected_tags=selected_tags
                        filter_mode=filter_mode
                        sort_mode=sort_mode
                        editing_target=editing_target
                        set_editing_target=set_editing_target
                        memo_editing_target=memo_editing_target
                        set_memo_editing_target=set_memo_editing_target
                    />
                    
                    <p class="item-count">{move || format!("{} items, {} tags", items.get().len(), tags.get().len())}</p>
                }>
                    <FilesWorkspace 
                        set_selected_file=set_selected_file 
                        set_editing_target=set_editing_target
                        selected_tags=selected_tags
                        filter_mode=filter_mode
                    />
                </Show>
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
            
            // Sync Configuration Modal
            <SyncModal
                show=show_sync_modal.into()
                set_show=set_show_sync_modal
                sync_url=sync_url.into()
                set_sync_url=set_sync_url
                sync_token=sync_token.into()
                set_sync_token=set_sync_token
                sync_status=sync_status.into()
                sync_msg=sync_msg.into()
                on_save_config=Callback::new(save_config)
                on_manual_sync=Callback::new(perform_manual_sync)
            />
        </div>
    }.into_any()
}
