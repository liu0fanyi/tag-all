//! Tag Editor Column
//!
//! Third column for editing item/tag properties with name edit, type selector, and tag management.

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::models::Tag;
use crate::commands::{self, CreateTagArgs};
use crate::context::AppContext;
use crate::components::tag_column::EditTarget;
use crate::components::type_selector::TypeSelector;
use crate::components::tag_autocomplete::TagAutocomplete;

/// Tag editor column (third column)
#[component]
pub fn TagEditor(
    editing_target: ReadSignal<Option<EditTarget>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
    // Name editing
    let (name_value, set_name_value) = signal(String::new());
    let (item_type, set_item_type) = signal(String::from("daily"));
    let (countdown_count, set_countdown_count) = signal(0i32);
    
    // Tags
    let (current_tags, set_current_tags) = signal(Vec::<Tag>::new());
    let (all_tags, set_all_tags) = signal(Vec::<Tag>::new());
    
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
    
    // Load current item/tag data when editing target changes OR reload happens
    Effect::new(move |_| {
        // Listen to both editing_target and reload_trigger
        let _ = ctx.reload_trigger.get();
        if let Some(target) = editing_target.get() {
            let current_target = match &target {
                EditTarget::Item(id, _) => Some((true, *id)),
                EditTarget::Tag(id, _) => Some((false, *id)),
            };
            
            // Only reset name if target changed
            let target_changed = last_target_id.get() != current_target;
            if target_changed {
                set_last_target_id.set(current_target);
            }
            
            match &target {
                EditTarget::Item(id, name) => {
                    if target_changed {
                        set_name_value.set(name.clone());
                    }
                    let id = *id;
                    spawn_local(async move {
                        // Get item type and count
                        if let Ok(Some(item)) = commands::get_item(id).await {
                            set_item_type.set(item.item_type.clone());
                            set_countdown_count.set(item.current_count);
                        }
                        // Get tags
                        if let Ok(tags) = commands::get_item_tags(id).await {
                            set_current_tags.set(tags);
                        }
                    });
                }
                EditTarget::Tag(id, name) => {
                    if target_changed {
                        set_name_value.set(name.clone());
                    }
                    set_item_type.set(String::new()); // Not applicable for tags
                    let id = *id;
                    spawn_local(async move {
                        if let Ok(tags) = commands::get_tag_parents(id).await {
                            set_current_tags.set(tags);
                        }
                    });
                }
            }
        } else {
            set_last_target_id.set(None);
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
                    let _ = commands::update_item(*id, Some(&name), None).await;
                }
                EditTarget::Tag(id, _) => {
                    let _ = commands::update_tag(*id, Some(&name), None).await;
                }
            }
            ctx.reload();
        });
    };
    
    // Save type (items only)
    let save_type = move |new_type: String| {
        let target = editing_target.get();
        if let Some(EditTarget::Item(id, _)) = target {
            spawn_local(async move {
                let _ = commands::update_item(id, None, Some(&new_type)).await;
                ctx.reload();
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
            let existing = all.iter().find(|t| t.name.to_lowercase() == name.to_lowercase());
            
            let tag_id = if let Some(tag) = existing {
                tag.id
            } else {
                // Create new tag
                let args = CreateTagArgs {
                    name: &name,
                    color: None,
                };
                match commands::create_tag(&args).await {
                    Ok(new_tag) => new_tag.id,
                    Err(_) => return,
                }
            };
            
            // Link tag to target
            match &target {
                EditTarget::Item(id, _) => {
                    let _ = commands::add_item_tag(*id, tag_id).await;
                }
                EditTarget::Tag(id, _) => {
                    if *id != tag_id {
                        let _ = commands::add_tag_parent(*id, tag_id).await;
                    }
                }
            }
            
            ctx.reload();
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
                }
            }
            ctx.reload();
        });
    };
    
    // Click on suggestion
    let on_suggestion_click = move |tag: Tag| {
        add_tag_by_name(tag.name);
    };

    view! {
        {move || match editing_target.get() {
            Some(target) => {
                let is_item = matches!(&target, EditTarget::Item(_, _));
                let title = match &target {
                    EditTarget::Item(_, _) => "编辑 Item",
                    EditTarget::Tag(_, _) => "编辑 Tag",
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
                                                            let _ = commands::set_item_count(id, Some(value)).await;
                                                            ctx.reload();
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
                                on_select=add_tag_by_name
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
