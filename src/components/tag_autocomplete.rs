//! Tag Autocomplete Component
//!
//! Reusable tag input with fuzzy search and autocomplete suggestions.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::models::Tag;

/// Simple fuzzy match: check if query chars appear in order in the target
pub fn fuzzy_match(query: &str, target: &str) -> bool {
    let query = query.to_lowercase();
    let target = target.to_lowercase();
    
    let mut target_chars = target.chars();
    for query_char in query.chars() {
        loop {
            match target_chars.next() {
                Some(c) if c == query_char => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}

/// Tag autocomplete input with suggestions
/// 
/// Props:
/// - all_tags: Signal containing all available tags for autocomplete
/// - on_select: Callback when a tag name is submitted
#[component]
pub fn TagAutocomplete(
    all_tags: ReadSignal<Vec<Tag>>,
    #[prop(into)] on_select: Callback<String>,
) -> impl IntoView {
    let (input_value, set_input_value) = signal(String::new());
    let (selected_idx, set_selected_idx) = signal(0usize);
    
    // Compute suggestions
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
    
    // Handle selection
    let handle_select = move |name: String| {
        on_select.run(name);
        set_input_value.set(String::new());
        set_selected_idx.set(0);
    };
    
    // Handle form submit
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let sugg = suggestions();
        let sel = selected_idx.get();
        
        let name = if !sugg.is_empty() && sel < sugg.len() {
            sugg[sel].name.clone()
        } else {
            input_value.get().trim().to_string()
        };
        
        if !name.is_empty() {
            handle_select(name);
        }
    };
    
    // Handle keydown
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let sugg = suggestions();
        
        match key.as_str() {
            "Tab" => {
                ev.prevent_default();
                if !sugg.is_empty() {
                    let sel = selected_idx.get();
                    if sel < sugg.len() {
                        set_input_value.set(sugg[sel].name.clone());
                    }
                }
            }
            "ArrowDown" => {
                ev.prevent_default();
                let sel = selected_idx.get();
                if sel + 1 < sugg.len() {
                    set_selected_idx.set(sel + 1);
                }
            }
            "ArrowUp" => {
                ev.prevent_default();
                let sel = selected_idx.get();
                if sel > 0 {
                    set_selected_idx.set(sel - 1);
                }
            }
            _ => {}
        }
    };
    
    view! {
        <div class="tag-input-wrapper">
            <form class="tag-editor-form" on:submit=on_submit>
                <input
                    type="text"
                    placeholder="输入标签名..."
                    autocomplete="off"
                    prop:value=move || input_value.get()
                    on:input=move |ev| {
                        let target = ev.target().unwrap();
                        let input = target.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                        set_input_value.set(input.value());
                        set_selected_idx.set(0);
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
                    let selected = selected_idx.get();
                    view! {
                        <div class="tag-suggestions">
                            {sugg.into_iter().enumerate().map(|(i, tag)| {
                                let name = tag.name.clone();
                                let name_for_click = name.clone();
                                let is_selected = i == selected;
                                view! {
                                    <button
                                        type="button"
                                        class=if is_selected { "suggestion-item selected" } else { "suggestion-item" }
                                        on:click=move |ev| {
                                            ev.prevent_default();
                                            handle_select(name_for_click.clone());
                                        }
                                    >
                                        {name}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
