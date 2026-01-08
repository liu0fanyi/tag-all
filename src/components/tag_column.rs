//! Tag Column Component
//!
//! Left sidebar displaying tag tree hierarchy with add input and DnD support.

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::models::Tag;
use crate::commands::{self, CreateTagArgs};
use crate::context::AppContext;
use crate::components::DeleteConfirmButton;
use crate::components::EditTarget;
use crate::store::{use_app_store, store_remove_tag, AppStateStoreFields};

use leptos_dragdrop::*;

/// Tag DnD Context - passed to all tag components via Leptos context
#[derive(Clone, Copy)]
pub struct TagDndContext {
    pub dnd: DndSignals,
    /// The parent tag ID of the currently dragged child (None = root tag)
    pub dragging_parent_id: ReadSignal<Option<u32>>,
    set_dragging_parent_id: WriteSignal<Option<u32>>,
}

impl TagDndContext {
    pub fn new() -> Self {
        let (dragging_parent_id, set_dragging_parent_id) = signal(None::<u32>);
        Self {
            dnd: create_dnd_signals(),
            dragging_parent_id,
            set_dragging_parent_id,
        }
    }
    
    pub fn start_drag(&self, tag_id: u32, parent_id: Option<u32>) {
        self.dnd.dragging_id_write.set(Some(tag_id));
        self.set_dragging_parent_id.set(parent_id);
    }
}

/// Tag add input
#[component]
fn TagAddInput() -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    let store = use_app_store();
    
    let (new_tag_name, set_new_tag_name) = signal(String::new());

    let add_tag = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name = new_tag_name.get();
        if name.is_empty() { return; }
        
        spawn_local(async move {
            let args = CreateTagArgs {
                name: &name,
                color: None,
            };
            if let Ok(new_tag) = commands::create_tag(&args).await {
                set_new_tag_name.set(String::new());
                // Fine-grained update: push new tag to store.root_tags
                store.root_tags().write().push(new_tag);
            }
        });
    };

    view! {
        <form class="tag-add-form" on:submit=add_tag>
            <input
                type="text"
                placeholder="Add tag..."
                prop:value=move || new_tag_name.get()
                on:input=move |ev| {
                    let target = ev.target().unwrap();
                    let input = target.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                    set_new_tag_name.set(input.value());
                }
            />
            <button type="submit">"+"</button>
        </form>
    }
}

/// Tag drop zone component
#[component]
fn TagDropZone(
    parent_id: Option<u32>,
    position: i32,
) -> impl IntoView {
    let tag_dnd = use_context::<TagDndContext>().expect("TagDndContext should be provided");
    let dnd = tag_dnd.dnd;
    
    let on_mouseenter = make_on_zone_mouseenter(dnd.clone(), parent_id, position);
    let on_mouseleave = make_on_mouseleave(dnd.clone());
    
    let is_active = move || {
        matches!(dnd.drop_target_read.get(), Some(DropTarget::Zone(pid, pos)) if pid == parent_id && pos == position)
    };
    
    let is_dragging = move || dnd.dragging_id_read.get().is_some();
    
    let zone_class = move || {
        let mut c = String::from("drop-zone tag-drop-zone");
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

/// Recursive tag tree item with DnD support
#[component]
fn TagTreeNode(
    tag: Tag,
    depth: usize,
    #[prop(into)] parent_id: Option<u32>,
    selected_tags: ReadSignal<Vec<u32>>,
    set_selected_tags: WriteSignal<Vec<u32>>,
    editing_target: ReadSignal<Option<EditTarget>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
    set_memo_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let id = tag.id;
    let position = tag.position;
    let name = tag.name.clone();
    let color = tag.color.clone().unwrap_or_else(|| "#666".to_string());
    let indent = depth * 16;
    
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    let store = use_app_store();
    let tag_dnd = use_context::<TagDndContext>().expect("TagDndContext should be provided");
    let dnd = tag_dnd.dnd;
    
    // Load children
    let (children, set_children) = signal(Vec::<Tag>::new());
    let (expanded, set_expanded) = signal(true);
    
    // Debounce for contextmenu to prevent duplicate events
    let (last_click_time, set_last_click_time) = signal(0f64);
    
    Effect::new(move |_| {
        // Watch store.tags_relation_version for changes
        let _ = store.tags_relation_version().get();
        spawn_local(async move {
            if let Ok(child_tags) = commands::get_tag_children(id).await {
                set_children.set(child_tags);
            }
        });
    });
    
    let is_selected = move || selected_tags.get().contains(&id);
    let has_children = move || !children.get().is_empty();
    
    // DnD handlers - use unified make_on_mousedown
    let on_mousedown = make_on_mousedown(dnd, id);
    let on_mouseenter = make_on_item_mouseenter(dnd, id);
    let on_mouseleave = make_on_mouseleave(dnd);
    
    let is_dragging = move || dnd.dragging_id_read.get() == Some(id);
    let is_drop_target = move || {
        matches!(dnd.drop_target_read.get(), Some(DropTarget::Item(tid)) if tid == id)
    };
    
    // Left-click handler - toggle tag selection for filtering (shift for multi-select)
    let on_click = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        
        let shift_held = ev.shift_key();
        let mut current_tags = selected_tags.get();
        
        if shift_held {
            // Shift+click: toggle this tag in selection
            if current_tags.contains(&id) {
                current_tags.retain(|&t| t != id);
            } else {
                current_tags.push(id);
            }
            set_selected_tags.set(current_tags);
        } else {
            // Normal click: single select (or deselect if already only selected)
            if current_tags == vec![id] {
                set_selected_tags.set(Vec::new()); // Deselect
            } else {
                set_selected_tags.set(vec![id]); // Select only this
            }
        }
    };
    
    // Right-click handler - opens properties editor
    let name_for_menu = name.clone();
    let on_context_menu = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        
        // Debounce: ignore events within 100ms
        let now = js_sys::Date::now();
        let last = last_click_time.get();
        if now - last < 100.0 {
            return;
        }
        set_last_click_time.set(now);
        
        // Close memo editor (Tags don't have memos)
        set_memo_editing_target.set(None);
        
        // Toggle properties editor
        let current = editing_target.get();
        let is_editing_this = matches!(&current, Some(EditTarget::Tag(tid, _)) if *tid == id);
        if is_editing_this {
            set_editing_target.set(None);
            spawn_local(async {
                let _ = commands::shrink_window(800, 700).await;
            });
        } else {
            set_editing_target.set(Some(EditTarget::Tag(id, name_for_menu.clone())));
            spawn_local(async {
                let _ = commands::resize_window(1100, 700).await;
            });
        }
    };
    
    let row_class = move || {
        let mut c = String::from("tag-tree-row");
        if is_selected() { c.push_str(" selected"); }
        if is_dragging() { c.push_str(" dragging"); }
        if is_drop_target() { c.push_str(" drop-target"); }
        c
    };

    view! {
        <div class="tag-tree-item">
            <div
                class=row_class
                style=format!("padding-left: {}px;", indent + 8)
                on:mousedown=on_mousedown
                on:mouseenter=on_mouseenter
                on:mouseleave=on_mouseleave
                on:click=on_click
                on:contextmenu=on_context_menu
            >
                {move || if has_children() {
                    view! {
                        <button
                            class="tag-expand-btn"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                set_expanded.update(|v| *v = !*v);
                            }
                        >
                            {move || if expanded.get() { "▼" } else { "▶" }}
                        </button>
                    }.into_any()
                } else {
                    view! { <span class="tag-expand-placeholder">"·"</span> }.into_any()
                }}
                
                <span class="tag-color-dot" style=format!("background-color: {};", color)></span>
                <span class="tag-tree-name">{format!("[{}] {}", position, tag.name)}</span>
                
                // Delete button with confirmation
                <DeleteConfirmButton
                    button_class="tag-delete-btn"
                    on_confirm=move || {
                        spawn_local(async move {
                            let _ = commands::delete_tag(id).await;
                            store_remove_tag(&store, id);
                        });
                    }
                />
            </div>
            
            // Children with drop zones
            {move || if expanded.get() {
                view! {
                    <div class="tag-tree-children">
                        <For
                            each=move || children.get()
                            key=|child| {
                                // Include position and name in key so component re-renders when they change
                                (child.id, child.position, child.name.clone())
                            }
                            children=move |child| {
                                let child_pos = child.position;
                                view! {
                                    // Drop zone before this child
                                    <TagDropZone parent_id=Some(id) position=child_pos />
                                    
                                    <TagTreeNode
                                        tag=child
                                        depth=depth + 1
                                        parent_id=Some(id)
                                        selected_tags=selected_tags
                                        set_selected_tags=set_selected_tags
                                        editing_target=editing_target
                                        set_editing_target=set_editing_target
                                        set_memo_editing_target=set_memo_editing_target
                                    />
                                }
                            }
                        />
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}

/// Tag column sidebar with DnD
#[component]
pub fn TagColumn(
    selected_tags: ReadSignal<Vec<u32>>,
    set_selected_tags: WriteSignal<Vec<u32>>,
    editing_target: ReadSignal<Option<EditTarget>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
    set_memo_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    let store = use_app_store();
    
    // Expect DnD context to be provided by parent (App)
    let tag_dnd = use_context::<TagDndContext>().expect("TagDndContext should be provided");
    let dnd = tag_dnd.dnd;
    
    // Bind global mouseup handler for dropping
    let ctx_for_drop = ctx; // used in closure if needed
    let dragging_parent = tag_dnd.dragging_parent_id;
    
    // We bind global mouseup here. Note: If App provides context, App could bind it,
    // but TagColumn is the main consumer/manager of Tag DnD logic.
    // However, since we now support dropping on Files (which are outside TagColumn),
    // having the logic here is still fine as long as we handle it.
    
    bind_global_mouseup(dnd.clone(), move |dragged_id, target| {
        let parent_id_when_dragged = dragging_parent.get_untracked();
        
        web_sys::console::log_1(&format!("[DnD] Drop: dragged_id={}, target={:?}, parent_when_dragged={:?}", 
            dragged_id, target, parent_id_when_dragged).into());
        
        spawn_local(async move {
            match target {
                DropTarget::Item(target_tag_id) => {
                    // Tag dropped on Tag = make dragged tag a child of target tag
                    web_sys::console::log_1(&format!("[DnD] Tag->Tag: {} becomes child of {}", dragged_id, target_tag_id).into());
                    if dragged_id != target_tag_id {
                        let _ = commands::add_tag_parent(dragged_id, target_tag_id).await;
                    }
                    // Refetch root_tags and update store
                    if let Ok(loaded) = commands::get_root_tags().await {
                        *store.root_tags().write() = loaded;
                    }
                    *store.tags_relation_version().write() += 1;
                }
                DropTarget::Zone(target_parent_id, position) => {
                    web_sys::console::log_1(&format!("[DnD] Zone drop: dragged={}, target_parent={:?}, position={}", 
                        dragged_id, target_parent_id, position).into());
                    // Determine if this is root tag or child tag
                    if target_parent_id.is_none() && parent_id_when_dragged.is_none() {
                        // Root tag moving within root
                        let _ = commands::move_tag(dragged_id, position).await;
                    } else if let Some(parent_id) = target_parent_id {
                        // Child tag moving within parent
                        let _ = commands::move_child_tag(dragged_id, parent_id, position).await;
                    }
                    // Refetch root_tags and update store
                    if let Ok(loaded) = commands::get_root_tags().await {
                        *store.root_tags().write() = loaded;
                    }
                    *store.tags_relation_version().write() += 1;
                }
                DropTarget::File(path) => {
                    web_sys::console::log_1(&format!("[DnD] Tag->File: {} dropped on {}", dragged_id, path).into());
                    // 1. Ensure file item exists
                    match commands::ensure_file_item(&path).await {
                        Ok(item) => {
                            // 2. Add tag to item
                            if let Ok(_) = commands::add_item_tag(item.id, dragged_id).await {
                                // 3. Trigger reload of Item List (if open)
                                // We can use the reload_trigger from AppContext
                                ctx_for_drop.reload();
                            }
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("Failed to ensure file item: {}", e).into());
                        }
                    }
                }
            }
        });
    });

    view! {
        <div class="tag-column">
            <div class="tag-column-header">"Tags"</div>
            
            <TagAddInput />
            
            <div class="tag-tree">
                <For
                    each=move || store.root_tags().get()
                    key=|tag| {
                        // Include position and name in key so component re-renders when they change
                        (tag.id, tag.position, tag.name.clone())
                    }
                    children=move |tag| {
                        let position = tag.position;
                        view! {
                            // Drop zone before this root tag
                            <TagDropZone parent_id=None position=position />
                            
                            <TagTreeNode
                                tag=tag
                                depth=0
                                parent_id=None
                                selected_tags=selected_tags
                                set_selected_tags=set_selected_tags
                                editing_target=editing_target
                                set_editing_target=set_editing_target
                                set_memo_editing_target=set_memo_editing_target
                            />
                        }
                    }
                />
            </div>
            
            {move || if store.root_tags().get().is_empty() {
                view! { <div class="no-tags-message">"No tags yet"</div> }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}
