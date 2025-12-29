//! Type Selector Component
//!
//! Reusable item type selector buttons.

use leptos::prelude::*;

/// Item type options
pub const ITEM_TYPES: &[(&str, &str)] = &[
    ("daily", "循环"),
    ("once", "一次性"),
    ("countdown", "倒数"),
    ("document", "文档"),
];

/// Type selector buttons for items
#[component]
pub fn TypeSelector(
    current_type: ReadSignal<String>,
    on_change: impl Fn(String) + Copy + 'static,
) -> impl IntoView {
    view! {
        <div class="type-selector">
            {ITEM_TYPES.iter().map(|(value, label)| {
                let val = value.to_string();
                let val_clone = val.clone();
                let is_selected = move || current_type.get() == val;
                view! {
                    <button
                        class=move || if is_selected() { "type-btn active" } else { "type-btn" }
                        on:click=move |_| on_change(val_clone.clone())
                    >
                        {*label}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}
