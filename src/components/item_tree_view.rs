//! Item Tree View Component
//!
//! Displays items in a tree structure with drag-and-drop support.
//! Uses leptos-dragdrop with explicit DropZones between items.

use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashSet;

use crate::models::Item;
use crate::tree::{flatten_tree, flatten_tree_sorted, TreeSortMode};
use crate::commands;
use crate::context::AppContext;
use crate::components::{TreeItem, EditTarget};
use crate::app::{FilterMode, SortMode};
use crate::store::{use_app_store, AppStateStoreFields};

use leptos_dragdrop::*;

/// Item tree view component with DnD support and tag filtering
#[component]
pub fn ItemTreeView(
    items: Memo<Vec<Item>>,
    selected_item: ReadSignal<Option<u32>>,
    set_selected_item: WriteSignal<Option<u32>>,
    selected_tags: ReadSignal<Vec<u32>>,
    filter_mode: ReadSignal<FilterMode>,
    sort_mode: ReadSignal<SortMode>,
    editing_target: ReadSignal<Option<EditTarget>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
    memo_editing_target: ReadSignal<Option<EditTarget>>,
    set_memo_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
    // Create DnD signals
    let dnd = create_dnd_signals();
    
    // Bind global mouseup handler for dropping
    let ws_id = ctx.current_workspace;
    let store = use_app_store();
    bind_global_mouseup(dnd.clone(), move |dragged_id, target| {
        spawn_local(async move {
            match target {
                DropTarget::Item(target_id) => {
                    let _ = commands::move_item(dragged_id, Some(target_id), 0).await;
                }
                DropTarget::Zone(parent_id, position) => {
                    let _ = commands::move_item(dragged_id, parent_id, position).await;
                }
            }
            // Refetch items and update store
            if let Ok(loaded) = commands::list_items_by_workspace(ws_id.get_untracked()).await {
                *store.items().write() = loaded;
            }
        });
    });
    
    // Item tags cache: stores (tag_ids, sorted_tag_names) for filtering and sorting
    let (item_tags_cache, set_item_tags_cache) = signal(std::collections::HashMap::<u32, (Vec<u32>, Vec<String>)>::new());
    
    // Load item tags when items change or when using tag-based features
    Effect::new(move |_| {
        let current_items = items.get();
        let selected = selected_tags.get();
        let sort = sort_mode.get();
        
        // Load tags if we have a filter OR using TagFirst sort mode
        if !selected.is_empty() || sort == SortMode::TagFirst {
            for item in current_items.iter() {
                let item_id = item.id;
                spawn_local(async move {
                    if let Ok(tags) = commands::get_item_tags(item_id).await {
                        // Backend sorts by pinyin
                        let tag_ids: Vec<u32> = tags.iter().map(|t| t.id).collect();
                        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
                        set_item_tags_cache.update(|cache| {
                            cache.insert(item_id, (tag_ids, tag_names));
                        });
                    }
                });
            }
        }
    });
    
    // Filtered, sorted and flattened tree items
    let tree_items = move || {
        let mut all_items = items.get();
        let selected = selected_tags.get();
        let mode = filter_mode.get();
        let sort = sort_mode.get();
        let cache = item_tags_cache.get();
        
        // Determine tree sort mode based on app sort mode
        let tree_sort = if sort == SortMode::NameFirst {
            // Pre-sort items by completed status then name
            all_items.sort_by(|a, b| {
                match (a.completed, b.completed) {
                    (false, true) => std::cmp::Ordering::Less,
                    (true, false) => std::cmp::Ordering::Greater,
                    _ => a.text.cmp(&b.text),
                }
            });
            TreeSortMode::Preserve
        } else if sort == SortMode::TagFirst {
            // Pre-sort items by first tag name
            all_items.sort_by(|a, b| {
                let a_tag = cache.get(&a.id).and_then(|(_, names)| names.first()).cloned().unwrap_or_default();
                let b_tag = cache.get(&b.id).and_then(|(_, names)| names.first()).cloned().unwrap_or_default();
                a_tag.cmp(&b_tag)
            });
            TreeSortMode::Preserve
        } else {
            TreeSortMode::Position
        };
        
        // If no tags selected, show all items
        if selected.is_empty() {
            return flatten_tree_sorted(&all_items, tree_sort);
        }
        
        let selected_set: HashSet<u32> = selected.into_iter().collect();
        
        // Filter items based on their tags
        let filtered: Vec<Item> = all_items.into_iter().filter(|item| {
            if let Some((item_tag_ids, _)) = cache.get(&item.id) {
                let item_tags: HashSet<u32> = item_tag_ids.iter().cloned().collect();
                match mode {
                    FilterMode::And => selected_set.is_subset(&item_tags),
                    FilterMode::Or => !selected_set.is_disjoint(&item_tags),
                }
            } else {
                false // Tag info not loaded yet, hide item
            }
        }).collect();
        
        flatten_tree_sorted(&filtered, tree_sort)
    };

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
                key=move |(item, depth)| {
                    // Include has_children in key so parent re-renders when child added
                    let has_children = items.get().iter().any(|i| i.parent_id == Some(item.id));
                    (
                        item.id,
                        *depth,
                        item.text.clone(),
                        item.item_type.clone(),
                        item.completed,
                        item.current_count,
                        item.position,
                        item.parent_id,
                        has_children, // NEW: triggers re-render when children change
                    )
                }
                children=move |(item, depth)| {
                    let id = item.id;
                    let parent_id = item.parent_id;
                    let position = item.position;
                    let has_children = items.get().iter().any(|i| i.parent_id == Some(id));
                    let is_selected = move || selected_item.get() == Some(id);
                    
                    // DnD handlers
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
                                memo_editing_target=memo_editing_target
                                set_memo_editing_target=set_memo_editing_target
                                set_selected_item=set_selected_item
                            />
                        </div>
                        
                        // Drop zone after this item
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
    
    let is_active = move || {
        matches!(dnd.drop_target_read.get(), Some(DropTarget::Zone(pid, pos)) if pid == parent_id && pos == position)
    };
    
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
