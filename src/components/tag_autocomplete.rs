//! Tag Autocomplete Component
//!
//! Reusable tag input with fuzzy search and autocomplete suggestions.
//! Supports semicolon-separated batch input for pasting multiple tags.

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

/// Get the current search segment (text after the last semicolon)
fn get_current_segment(input: &str) -> &str {
    input.rsplit(';').next().unwrap_or("").trim()
}

/// Replace the current segment (after last semicolon) with a new value
fn replace_current_segment(input: &str, new_segment: &str) -> String {
    if let Some(pos) = input.rfind(';') {
        format!("{}; {}", &input[..pos], new_segment)
    } else {
        new_segment.to_string()
    }
}

/// Tag autocomplete input with suggestions
/// 
/// Props:
/// - all_tags: Signal containing all available tags for autocomplete
/// - on_select: Callback when a tag name is submitted (called for EACH tag)
#[component]
pub fn TagAutocomplete(
    all_tags: ReadSignal<Vec<Tag>>,
    #[prop(into)] on_select: Callback<String>,
) -> impl IntoView {
    let (input_value, set_input_value) = signal(String::new());
    let (selected_idx, set_selected_idx) = signal(0usize);
    
    // Compute suggestions based on current segment (after last semicolon)
    let suggestions = move || {
        let full_input = input_value.get();
        let current_segment = get_current_segment(&full_input);
        
        if current_segment.is_empty() {
            return vec![];
        }
        
        all_tags.get()
            .into_iter()
            .filter(|tag| fuzzy_match(current_segment, &tag.name))
            .take(5)
            .collect::<Vec<_>>()
    };
    
    // Handle selecting a suggestion - replaces only current segment
    let handle_suggestion_select = move |name: String| {
        let full_input = input_value.get();
        let new_input = replace_current_segment(&full_input, &name);
        set_input_value.set(new_input);
        set_selected_idx.set(0);
    };
    
    // Handle form submit - processes ALL semicolon-separated tags
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let full_input = input_value.get();
        
        // Split by semicolon and add each tag
        let tags: Vec<String> = full_input
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // If we have suggestions and the current segment matches, use suggestion
        let sugg = suggestions();
        let sel = selected_idx.get();
        
        for (i, tag_name) in tags.iter().enumerate() {
            // For the last tag, check if we should use the suggestion
            let final_name = if i == tags.len() - 1 && !sugg.is_empty() && sel < sugg.len() {
                // Check if current segment partially matches the suggestion
                if fuzzy_match(tag_name, &sugg[sel].name) {
                    sugg[sel].name.clone()
                } else {
                    tag_name.clone()
                }
            } else {
                tag_name.clone()
            };
            
            if !final_name.is_empty() {
                on_select.run(final_name);
            }
        }
        
        // Clear input
        set_input_value.set(String::new());
        set_selected_idx.set(0);
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
                        // Replace only the current segment with the suggestion
                        handle_suggestion_select(sugg[sel].name.clone());
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
                    placeholder="输入标签名 (分号分隔多个)..."
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
            
            // Autocomplete suggestions (for current segment only)
            {move || {
                let sugg = suggestions();
                if sugg.is_empty() {
                    view! { <div></div> }.into_any()
                } else {
                    let selected = selected_idx.get();
                    view! {
                        <div class="autocomplete-list">
                            {sugg.into_iter().enumerate().map(|(i, tag)| {
                                let name = tag.name.clone();
                                let name_for_click = name.clone();
                                let is_selected = i == selected;
                                view! {
                                    <button
                                        type="button"
                                        class=if is_selected { "autocomplete-item selected" } else { "autocomplete-item" }
                                        on:click=move |ev| {
                                            ev.prevent_default();
                                            handle_suggestion_select(name_for_click.clone());
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
