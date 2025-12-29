//! Drop Zone Component
//!
//! A horizontal line drop zone between items for drag-and-drop reordering.

use leptos::prelude::*;
use web_sys::DragEvent;

/// Drop zone component shown between items to indicate drop position
#[component]
pub fn DropZone(
    /// Parent ID where item will be placed (None = root)
    parent_id: Option<u32>,
    /// Position index where item will be inserted
    position: i32,
    /// Callback when item is dropped here
    on_drop: Callback<(u32, Option<u32>, i32)>,
    /// Currently dragging item ID
    dragging_id: ReadSignal<Option<u32>>,
) -> impl IntoView {
    let (is_over, set_is_over) = signal(false);
    
    let on_dragover = move |ev: DragEvent| {
        ev.prevent_default();
        set_is_over.set(true);
    };
    
    let on_dragleave = move |_: DragEvent| {
        set_is_over.set(false);
    };
    
    let on_drop_handler = move |ev: DragEvent| {
        ev.prevent_default();
        set_is_over.set(false);
        
        if let Some(id) = dragging_id.get_untracked() {
            on_drop.run((id, parent_id, position));
        }
    };
    
    // Only show when dragging
    let is_visible = move || dragging_id.get().is_some();
    
    view! {
        <div
            class=move || {
                let mut c = "drop-zone".to_string();
                if is_over.get() { c.push_str(" active"); }
                if !is_visible() { c.push_str(" hidden"); }
                c
            }
            on:dragover=on_dragover
            on:dragleave=on_dragleave
            on:drop=on_drop_handler
        />
    }
}
