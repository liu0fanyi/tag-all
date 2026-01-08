//! Tag Editor Column
//!
//! Third column for editing item/tag properties with name edit, type selector, and tag management.

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::models::Tag;
use crate::commands::{self, CreateTagArgs};
use crate::context::AppContext;
use crate::components::EditTarget;
use crate::components::type_selector::TypeSelector;
use crate::components::tag_autocomplete::TagAutocomplete;
use crate::store::{use_app_store, store_update_item, store_update_tag, AppStateStoreFields};

/// Tag editor column (third column)
#[component]
pub fn TagEditor(
    editing_target: ReadSignal<Option<EditTarget>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    let store = use_app_store();
    
    // Name editing
    let (name_value, set_name_value) = signal(String::new());
    let (item_type, set_item_type) = signal(String::from("daily"));
    let (countdown_count, set_countdown_count) = signal(0i32);
    
    // Tags
    let (current_tags, set_current_tags) = signal(Vec::<Tag>::new());
    let (all_tags, set_all_tags) = signal(Vec::<Tag>::new());
    // Common tags across multi-selected items (intersection)
    let (common_tags, set_common_tags) = signal(Vec::<Tag>::new());
    
    // Load all tags for autocomplete
    Effect::new(move |_| {
        let _ = ctx.reload_trigger.get();
        spawn_local(async move {
            if let Ok(tags) = commands::list_tags().await {
                set_all_tags.set(tags);
            }
        });
    });
    
    // Track which target we're editing to avoid resetting name on reload
    let (last_target_id, set_last_target_id) = signal::<Option<(bool, u32)>>(None); // (is_item, id)
    // Track multi-item IDs
    let (last_multi_ids, set_last_multi_ids) = signal::<Vec<u32>>(Vec::new());
    
    // Load current item/tag data when editing target changes OR reload happens
    Effect::new(move |_| {
        // Listen to both editing_target and reload_trigger
        let _ = ctx.reload_trigger.get();
        if let Some(target) = editing_target.get() {
            match &target {
                EditTarget::Item(id, name) => {
                    let current_target = Some((true, *id));
                    let target_changed = last_target_id.get() != current_target;
                    if target_changed {
                        set_last_target_id.set(current_target);
                        set_name_value.set(name.clone());
                    }
                    set_last_multi_ids.set(Vec::new());
                    set_common_tags.set(Vec::new());
                    let id = *id;
                    spawn_local(async move {
                        // Get item type and count
                        if let Ok(Some(item)) = commands::get_item(id).await {
                            set_item_type.set(item.item_type.clone());
                            set_countdown_count.set(item.current_count);
                        }
                        // Get tags (backend sorts by pinyin)
                        if let Ok(tags) = commands::get_item_tags(id).await {
                            set_current_tags.set(tags);
                        }
                    });
                }
                EditTarget::Tag(id, name) => {
                    let current_target = Some((false, *id));
                    let target_changed = last_target_id.get() != current_target;
                    if target_changed {
                        set_last_target_id.set(current_target);
                        set_name_value.set(name.clone());
                    }
                    set_item_type.set(String::new()); // Not applicable for tags
                    set_last_multi_ids.set(Vec::new());
                    set_common_tags.set(Vec::new());
                    let id = *id;
                    spawn_local(async move {
                        if let Ok(tags) = commands::get_tag_parents(id).await {
                            // Backend sorts by pinyin
                            set_current_tags.set(tags);
                        }
                    });
                }
                EditTarget::MultiItems(ids) => {
                    set_last_target_id.set(None);
                    set_name_value.set(String::new());
                    set_item_type.set(String::new());
                    set_current_tags.set(Vec::new());
                    
                    // Check if target changed
                    let target_changed = last_multi_ids.get() != *ids;
                    if target_changed {
                        set_last_multi_ids.set(ids.clone());
                    }
                    
                    // Compute common tags intersection
                    let ids_cloned = ids.clone();
                    spawn_local(async move {
                        use std::collections::HashSet;
                        let mut common: Option<HashSet<u32>> = None;
                        let mut tag_map = std::collections::HashMap::<u32, Tag>::new();
                        
                        for item_id in ids_cloned.iter() {
                            if let Ok(tags) = commands::get_item_tags(*item_id).await {
                                let tag_ids: HashSet<u32> = tags.iter().map(|t| t.id).collect();
                                for tag in tags {
                                    tag_map.insert(tag.id, tag);
                                }
                                common = Some(match common {
                                    Some(c) => c.intersection(&tag_ids).cloned().collect(),
                                    None => tag_ids,
                                });
                            }
                        }
                        
                        let common_tag_ids = common.unwrap_or_default();
                        let common_tags_list: Vec<Tag> = common_tag_ids.iter()
                            .filter_map(|id| tag_map.get(id).cloned())
                            .collect();
                        set_common_tags.set(common_tags_list);
                    });
                }
            }
        } else {
            set_last_target_id.set(None);
            set_last_multi_ids.set(Vec::new());
            set_common_tags.set(Vec::new());
        }
    });
    
    // Save name
    let save_name = move || {
        let name = name_value.get().trim().to_string();
        if name.is_empty() { return; }
        
        let target = editing_target.get();
        if target.is_none() { return; }
        let target = target.unwrap();
        
        spawn_local(async move {
            match &target {
                EditTarget::Item(id, _) => {
                    if let Ok(updated) = commands::update_item(*id, Some(&name), None).await {
                        store_update_item(&store, updated);
                    }
                }
                EditTarget::Tag(id, _) => {
                    if let Ok(updated) = commands::update_tag(*id, Some(&name), None).await {
                        store_update_tag(&store, updated);
                        // Increment version to trigger child tag reload in TagTreeNode
                        *store.tags_relation_version().write() += 1;
                    }
                }
                EditTarget::MultiItems(_) => {
                    // Multi-items don't have a single name to save
                }
            }
        });
    };
    
    // Save type (items only)
    let save_type = move |new_type: String| {
        let target = editing_target.get();
        if let Some(EditTarget::Item(id, _)) = target {
            spawn_local(async move {
                if let Ok(updated) = commands::update_item(id, None, Some(&new_type)).await {
                    store_update_item(&store, updated);
                }
            });
        }
    };
    
    // Add tag (create if not exists) - called by TagAutocomplete on_select
    let add_tag_by_name = move |name: String| {
        if name.is_empty() { return; }
        
        let target = editing_target.get();
        if target.is_none() { return; }
        let target = target.unwrap();
        
        spawn_local(async move {
            // First try to find existing tag by name
            let all = commands::list_tags().await.unwrap_or_default();
            let existing = all.iter().find(|t| t.name.to_lowercase() == name.to_lowercase()).cloned();
            
            let tag = if let Some(tag) = existing {
                tag
            } else {
                // Create new tag
                let args = CreateTagArgs {
                    name: &name,
                    color: None,
                };
                match commands::create_tag(&args).await {
                    Ok(new_tag) => {
                        // Also add to store.root_tags for TagColumn
                        store.root_tags().write().push(new_tag.clone());
                        new_tag
                    }
                    Err(_) => return,
                }
            };
            
            // Link tag to target(s)
            match &target {
                EditTarget::Item(id, _) => {
                    let _ = commands::add_item_tag(*id, tag.id).await;
                }
                EditTarget::Tag(id, _) => {
                    if *id != tag.id {
                        let _ = commands::add_tag_parent(*id, tag.id).await;
                        // Refetch root_tags since tag hierarchy changed
                        if let Ok(loaded) = commands::get_root_tags().await {
                            *store.root_tags().write() = loaded;
                        }
                    }
                }
                EditTarget::MultiItems(ids) => {
                    // Batch add tag to all selected items
                    for item_id in ids.iter() {
                        let _ = commands::add_item_tag(*item_id, tag.id).await;
                    }
                }
            }
            
            // Fine-grained update: push to local current_tags (for single item/tag)
            set_current_tags.update(|tags| {
                if !tags.iter().any(|t| t.id == tag.id) {
                    tags.push(tag.clone());
                }
            });
            // For multi-items, also add to common_tags since it's now common to all
            set_common_tags.update(|tags| {
                if !tags.iter().any(|t| t.id == tag.id) {
                    tags.push(tag);
                }
            });
            // Trigger tag child reload
            *store.tags_relation_version().write() += 1;
        });
    };
    
    // Remove tag
    let remove_tag = move |tag_id: u32| {
        let target = editing_target.get();
        if target.is_none() { return; }
        let target = target.unwrap();
        
        spawn_local(async move {
            match &target {
                EditTarget::Item(id, _) => {
                    let _ = commands::remove_item_tag(*id, tag_id).await;
                }
                EditTarget::Tag(id, _) => {
                    let _ = commands::remove_tag_parent(*id, tag_id).await;
                    // Refetch root_tags since tag hierarchy changed
                    if let Ok(loaded) = commands::get_root_tags().await {
                        *store.root_tags().write() = loaded;
                    }
                }
                EditTarget::MultiItems(ids) => {
                    // Batch remove tag from all selected items
                    for item_id in ids.iter() {
                        let _ = commands::remove_item_tag(*item_id, tag_id).await;
                    }
                }
            }
            // Fine-grained update: remove from local current_tags
            set_current_tags.update(|tags| {
                tags.retain(|t| t.id != tag_id);
            });
            // Also remove from common_tags for multi-items
            set_common_tags.update(|tags| {
                tags.retain(|t| t.id != tag_id);
            });
            // Trigger tag child reload
            *store.tags_relation_version().write() += 1;
        });
    };
    
    // Click on suggestion
    let on_suggestion_click = move |tag: Tag| {
        add_tag_by_name(tag.name);
    };

    view! {
        {move || match editing_target.get() {
            Some(EditTarget::MultiItems(ids)) => {
                // Multi-select mode: tag-only editor with common tags
                let count = ids.len();
                view! {
                    <div class="tag-editor-column">
                        <div class="tag-editor-header">
                            <span class="tag-editor-title">{format!("编辑 {} 个项目", count)}</span>
                            <button class="close-btn" on:click=move |_| set_editing_target.set(None)>"×"</button>
                        </div>
                        
                        // Tag input section
                        <div class="editor-section">
                            <label class="editor-label">"添加标签"</label>
                            <TagAutocomplete 
                                all_tags=all_tags
                                on_select=add_tag_by_name.clone()
                            />
                        </div>
                        
                        // Common tags section (intersection of all selected items' tags)
                        <div class="editor-section common-tags-section">
                            <label class="editor-label">"共同标签"</label>
                            <div class="current-tags-list">
                                <For
                                    each=move || common_tags.get()
                                    key=|tag| tag.id
                                    children=move |tag| {
                                        let tag_id = tag.id;
                                        let color = tag.color.clone().unwrap_or_else(|| "#666".to_string());
                                        view! {
                                            <div class="current-tag-item">
                                                <span class="tag-color-dot" style=format!("background-color: {};", color)></span>
                                                <span class="current-tag-name">{tag.name}</span>
                                                <button class="remove-tag-btn" on:click=move |_| remove_tag(tag_id)>"×"</button>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                            <Show when=move || common_tags.get().is_empty()>
                                <p class="no-common-tags">"无共同标签"</p>
                            </Show>
                        </div>
                    </div>
                }.into_any()
            }
            Some(target) => {
                let is_item = matches!(&target, EditTarget::Item(_, _));
                let title = match &target {
                    EditTarget::Item(_, _) => "编辑 Item",
                    EditTarget::Tag(_, _) => "编辑 Tag",
                    _ => "",
                };
                
                view! {
                    <div class="tag-editor-column">
                        <div class="tag-editor-header">
                            <span class="tag-editor-title">{title}</span>
                            <button class="close-btn" on:click=move |_| set_editing_target.set(None)>"×"</button>
                        </div>
                        
                        // Name edit section
                        <div class="editor-section">
                            <label class="editor-label">"名称"</label>
                            <div class="name-edit-row">
                                <input
                                    type="text"
                                    class="name-edit-input"
                                    class:disabled-input=move || item_type.get() == "document"
                                    prop:disabled=move || item_type.get() == "document"
                                    title=move || if item_type.get() == "document" { "文件名称不可修改" } else { "" }
                                    prop:value=move || name_value.get()
                                    on:input=move |ev| {
                                        let target = ev.target().unwrap();
                                        let input = target.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                                        set_name_value.set(input.value());
                                    }
                                    on:blur=move |_| save_name()
                                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                                        if ev.key() == "Enter" {
                                            ev.prevent_default();
                                            save_name();
                                        }
                                    }
                                />
                            </div>
                        </div>
                        
                        // Item type selector (only for items)
                        {move || if is_item {
                            let save_type_fn = save_type.clone();
                            view! {
                                <div class="editor-section">
                                    <label class="editor-label">"类型"</label>
                                    <TypeSelector 
                                        current_type=item_type
                                        on_change=move |new_type: String| {
                                            set_item_type.set(new_type.clone());
                                            save_type_fn(new_type);
                                        }
                                    />
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
                        
                        // Countdown input section (only for countdown type items)
                        {move || {
                            let is_countdown = item_type.get() == "countdown";
                            if is_item && is_countdown {
                                let target_id = match editing_target.get() {
                                    Some(EditTarget::Item(id, _)) => Some(id),
                                    _ => None,
                                };
                                view! {
                                    <div class="editor-section">
                                        <label class="editor-label">"倒数初始值"</label>
                                        <div class="countdown-input-row">
                                            <input
                                                type="number"
                                                class="countdown-input"
                                                prop:value=move || countdown_count.get()
                                                on:change=move |ev| {
                                                    use wasm_bindgen::JsCast;
                                                    let target = ev.target().unwrap();
                                                    let input = target.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                                                    let value: i32 = input.value().parse().unwrap_or(0);
                                                    set_countdown_count.set(value);
                                                    
                                                    // Save to backend
                                                    if let Some(id) = target_id {
                                                        spawn_local(async move {
                                                            if let Ok(updated) = commands::set_item_count(id, Some(value)).await {
                                                                store_update_item(&store, updated);
                                                            }
                                                        });
                                                    }
                                                }
                                            />
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }}
                        
                        // Tag input section
                        <div class="editor-section">
                            <label class="editor-label">"添加标签"</label>
                            <TagAutocomplete 
                                all_tags=all_tags
                                on_select=add_tag_by_name.clone()
                            />
                        </div>
                        
                        // Current tags list
                        <div class="editor-section">
                            <label class="editor-label">"已添加的标签"</label>
                            <div class="current-tags-list">
                                <For
                                    each=move || current_tags.get()
                                    key=|tag| tag.id
                                    children=move |tag| {
                                        let tag_id = tag.id;
                                        let color = tag.color.clone().unwrap_or_else(|| "#666".to_string());
                                        view! {
                                            <div class="current-tag-item">
                                                <span class="tag-color-dot" style=format!("background-color: {};", color)></span>
                                                <span class="current-tag-name">{tag.name}</span>
                                                <button class="remove-tag-btn" on:click=move |_| remove_tag(tag_id)>"×"</button>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        </div>
                    </div>
                }.into_any()
            }
            None => view! { <div></div> }.into_any()
        }}
    }
}
