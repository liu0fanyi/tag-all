//! Item Tree View Component
//!
//! Displays items in a tree structure with drag-and-drop support.
//! Uses leptos-dragdrop with explicit DropZones between items.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::models::Item;
use crate::tree::flatten_tree;
use crate::commands;
use crate::context::AppContext;
use crate::components::{TreeItem, EditTarget};

use leptos_dragdrop::*;

/// Item tree view component with DnD support
#[component]
pub fn ItemTreeView(
    items: ReadSignal<Vec<Item>>,
    selected_item: ReadSignal<Option<u32>>,
    set_selected_item: WriteSignal<Option<u32>>,
    editing_target: ReadSignal<Option<EditTarget>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
    // Create DnD signals
    let dnd = create_dnd_signals();
    
    // Bind global mouseup handler for dropping
    // Note: we need to get the reload_trigger write signal to use in the async closure
    let set_reload = ctx.clone();
    bind_global_mouseup(dnd.clone(), move |dragged_id, target| {
        let set_reload = set_reload.clone();
        spawn_local(async move {
            match target {
                DropTarget::Item(target_id) => {
                    // Become child of target item (position 0)
                    web_sys::console::log_1(&format!("[DND] Drop on Item: dragged={}, target={}", dragged_id, target_id).into());
                    let _ = commands::move_item(dragged_id, Some(target_id), 0).await;
                }
                DropTarget::Zone(parent_id, position) => {
                    // Insert at specific position under parent
                    web_sys::console::log_1(&format!("[DND] Drop on Zone: dragged={}, parent={:?}, position={}", dragged_id, parent_id, position).into());
                    let _ = commands::move_item(dragged_id, parent_id, position).await;
                }
            }
            // Reload after backend update completes
            web_sys::console::log_1(&"[DND] Backend done, reloading...".into());
            set_reload.reload();
        });
    });
    
    let tree_items = move || flatten_tree(&items.get());

    view! {
        <div class="tree-view">
            // Initial drop zone at top (root level, position 0)
            <DropZone
                dnd=dnd.clone()
                parent_id=None
                position=0
            />
            
            <For
                each=tree_items
                key=|(item, depth)| {
                    // Use a tuple of all mutable fields to ensure changes cause re-render
                    // This is verbose but guaranteed to work
                    (
                        item.id,
                        *depth,
                        item.text.clone(),
                        item.item_type.clone(),
                        item.completed,
                        item.current_count,
                        item.position,
                        item.parent_id,
                    )
                }
                children=move |(item, depth)| {
                    let id = item.id;
                    let parent_id = item.parent_id;
                    let position = item.position;
                    let has_children = items.get().iter().any(|i| i.parent_id == Some(id));
                    let is_selected = move || selected_item.get() == Some(id);
                    
                    // DnD handlers - use unified make_on_mousedown
                    let on_mousedown = make_on_mousedown(dnd, id);
                    let on_mouseenter = make_on_item_mouseenter(dnd, id);
                    let on_mouseleave = make_on_mouseleave(dnd);
                    
                    // Visual state
                    let is_dragging = move || dnd.dragging_id_read.get() == Some(id);
                    let is_drop_target = move || {
                        matches!(dnd.drop_target_read.get(), Some(DropTarget::Item(tid)) if tid == id)
                    };
                    
                    let item_class = move || {
                        let mut c = String::from("tree-item-wrapper");
                        if is_selected() { c.push_str(" selected"); }
                        if is_dragging() { c.push_str(" dragging"); }
                        if is_drop_target() { c.push_str(" drop-target"); }
                        c
                    };
                    
                    view! {
                        <div
                            class=item_class
                            on:mousedown=on_mousedown
                            on:mouseenter=on_mouseenter
                            on:mouseleave=on_mouseleave
                            on:click=move |_| set_selected_item.set(Some(id))
                        >
                            <TreeItem
                                item=item.clone()
                                depth=depth
                                has_children=has_children
                                editing_target=editing_target
                                set_editing_target=set_editing_target
                                set_selected_item=set_selected_item
                            />
                        </div>
                        
                        // Drop zone after this item (same parent, position + 1)
                        <DropZone
                            dnd=dnd.clone()
                            parent_id=parent_id
                            position=position + 1
                        />
                    }
                }
            />
        </div>
    }
}

/// Drop zone component - a horizontal separator for dropping items
#[component]
pub fn DropZone(
    dnd: DndSignals,
    parent_id: Option<u32>,
    position: i32,
) -> impl IntoView {
    let on_mouseenter = make_on_zone_mouseenter(dnd.clone(), parent_id, position);
    let on_mouseleave = make_on_mouseleave(dnd.clone());
    
    // Is this zone the current drop target?
    let is_active = move || {
        matches!(dnd.drop_target_read.get(), Some(DropTarget::Zone(pid, pos)) if pid == parent_id && pos == position)
    };
    
    // Only show when dragging
    let is_dragging = move || dnd.dragging_id_read.get().is_some();
    
    let zone_class = move || {
        let mut c = String::from("drop-zone");
        if !is_dragging() { c.push_str(" hidden"); }
        if is_active() { c.push_str(" active"); }
        c
    };
    
    view! {
        <div
            class=zone_class
            on:mouseenter=on_mouseenter
            on:mouseleave=on_mouseleave
        />
    }
}
