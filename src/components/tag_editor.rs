//! Tag Editor Column
//!
//! Third column for editing tags on an item or tag, with fuzzy search autocomplete.

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::models::Tag;
use crate::commands::{self, CreateTagArgs};
use crate::context::AppContext;
use crate::components::tag_column::EditTarget;

/// Simple fuzzy match: check if query chars appear in order in the target
fn fuzzy_match(query: &str, target: &str) -> bool {
    let query = query.to_lowercase();
    let target = target.to_lowercase();
    
    let mut query_chars = query.chars().peekable();
    for c in target.chars() {
        if query_chars.peek() == Some(&c) {
            query_chars.next();
        }
        if query_chars.peek().is_none() {
            return true;
        }
    }
    query_chars.peek().is_none()
}

/// Tag editor column (third column)
#[component]
pub fn TagEditor(
    editing_target: ReadSignal<Option<EditTarget>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
    let (input_value, set_input_value) = signal(String::new());
    let (current_tags, set_current_tags) = signal(Vec::<Tag>::new());
    let (all_tags, set_all_tags) = signal(Vec::<Tag>::new());
    let (selected_suggestion, set_selected_suggestion) = signal(0usize);
    
    // Load all tags for autocomplete
    Effect::new(move |_| {
        let _ = ctx.reload_trigger.get();
        spawn_local(async move {
            if let Ok(tags) = commands::list_tags().await {
                set_all_tags.set(tags);
            }
        });
    });
    
    // Load current tags for the editing target
    Effect::new(move |_| {
        let _ = ctx.reload_trigger.get();
        if let Some(target) = editing_target.get() {
            spawn_local(async move {
                let tags = match &target {
                    EditTarget::Item(id, _) => commands::get_item_tags(*id).await,
                    EditTarget::Tag(id, _) => commands::get_tag_parents(*id).await,
                };
                if let Ok(tags) = tags {
                    set_current_tags.set(tags);
                }
            });
        }
    });
    
    // Compute suggestions based on input
    let suggestions = move || {
        let query = input_value.get();
        if query.is_empty() {
            return vec![];
        }
        
        all_tags.get()
            .into_iter()
            .filter(|tag| fuzzy_match(&query, &tag.name))
            .take(5)
            .collect::<Vec<_>>()
    };
    
    // Add tag (create if not exists)
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
            
            // Link tag to target (prevent self-reference for tags)
            match &target {
                EditTarget::Item(id, _) => {
                    let _ = commands::add_item_tag(*id, tag_id).await;
                }
                EditTarget::Tag(id, _) => {
                    // Prevent adding a tag as its own parent
                    if *id != tag_id {
                        let _ = commands::add_tag_parent(*id, tag_id).await;
                    }
                }
            }
            
            set_input_value.set(String::new());
            set_selected_suggestion.set(0);
            ctx.reload();
        });
    };
    
    // Handle form submit
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let sugg = suggestions();
        let sel = selected_suggestion.get();
        
        let name = if !sugg.is_empty() && sel < sugg.len() {
            sugg[sel].name.clone()
        } else {
            input_value.get().trim().to_string()
        };
        
        add_tag_by_name(name);
    };
    
    // Handle keydown for Tab and arrow keys
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let sugg = suggestions();
        
        match key.as_str() {
            "Tab" => {
                ev.prevent_default();
                if !sugg.is_empty() {
                    let sel = selected_suggestion.get();
                    if sel < sugg.len() {
                        set_input_value.set(sugg[sel].name.clone());
                    }
                }
            }
            "ArrowDown" => {
                ev.prevent_default();
                let sel = selected_suggestion.get();
                if sel + 1 < sugg.len() {
                    set_selected_suggestion.set(sel + 1);
                }
            }
            "ArrowUp" => {
                ev.prevent_default();
                let sel = selected_suggestion.get();
                if sel > 0 {
                    set_selected_suggestion.set(sel - 1);
                }
            }
            "Escape" => {
                set_input_value.set(String::new());
                set_selected_suggestion.set(0);
            }
            _ => {}
        }
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
                let title = match &target {
                    EditTarget::Item(_, name) => format!("编辑: {}", name),
                    EditTarget::Tag(_, name) => format!("编辑: {}", name),
                };
                
                view! {
                    <div class="tag-editor-column">
                        <div class="tag-editor-header">
                            <span class="tag-editor-title">{title}</span>
                            <button class="close-btn" on:click=move |_| set_editing_target.set(None)>"×"</button>
                        </div>
                        
                        <div class="tag-input-wrapper">
                            <form class="tag-editor-form" on:submit=on_submit>
                                <input
                                    type="text"
                                    placeholder="Add tag..."
                                    autocomplete="off"
                                    prop:value=move || input_value.get()
                                    on:input=move |ev| {
                                        let target = ev.target().unwrap();
                                        let input = target.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                                        set_input_value.set(input.value());
                                        set_selected_suggestion.set(0);
                                    }
                                    on:keydown=on_keydown
                                />
                                <button type="submit">"+"</button>
                            </form>
                            
                            // Autocomplete suggestions
                            {move || {
                                let sugg = suggestions();
                                if sugg.is_empty() {
                                    view! { <div></div> }.into_any()
                                } else {
                                    view! {
                                        <div class="autocomplete-list">
                                            {sugg.into_iter().enumerate().map(|(i, tag)| {
                                                let tag_clone = tag.clone();
                                                let is_selected = move || selected_suggestion.get() == i;
                                                let color = tag.color.clone().unwrap_or_else(|| "#666".to_string());
                                                view! {
                                                    <div
                                                        class=move || if is_selected() { "autocomplete-item selected" } else { "autocomplete-item" }
                                                        on:click=move |_| on_suggestion_click(tag_clone.clone())
                                                    >
                                                        <span class="tag-color-dot" style=format!("background-color: {};", color)></span>
                                                        <span>{tag.name}</span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                        
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
                }.into_any()
            }
            None => view! { <div></div> }.into_any()
        }}
    }
}
