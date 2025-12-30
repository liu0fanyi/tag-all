//! Delete Confirm Button Component
//!
//! Reusable inline delete confirmation button with confirm/cancel actions.

use leptos::prelude::*;

/// Inline delete confirmation button
/// 
/// Shows a × button initially. When clicked, shows "删除?" with ✓/✗ buttons.
/// 
/// # Arguments
/// * `button_class` - CSS class for the initial delete button (e.g., "delete-btn" or "tag-delete-btn")
/// * `on_confirm` - Callback to execute when user confirms deletion
#[component]
pub fn DeleteConfirmButton(
    #[prop(into)] button_class: String,
    #[prop(into)] on_confirm: Callback<()>,
) -> impl IntoView {
    let (confirm_delete, set_confirm_delete) = signal(false);
    
    view! {
        <Show when=move || !confirm_delete.get()>
            <button
                class=button_class.clone()
                on:click=move |ev| {
                    ev.stop_propagation();
                    set_confirm_delete.set(true);
                }
            >
                "×"
            </button>
        </Show>
        <Show when=move || confirm_delete.get()>
            <span class="delete-confirm">
                <span class="delete-confirm-text">"删除?"</span>
                <button
                    class="confirm-btn"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        on_confirm.run(());
                    }
                >
                    "✓"
                </button>
                <button
                    class="cancel-btn"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        set_confirm_delete.set(false);
                    }
                >
                    "✗"
                </button>
            </span>
        </Show>
    }
}
