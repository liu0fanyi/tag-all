//! Tag Column Component
//!
//! Left sidebar displaying tag tree hierarchy with add input.

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::models::Tag;
use crate::commands::{self, CreateTagArgs};
use crate::context::AppContext;

/// Tag add input
#[component]
fn TagAddInput() -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
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
            if commands::create_tag(&args).await.is_ok() {
                set_new_tag_name.set(String::new());
                ctx.reload();
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

/// Recursive tag tree item
#[component]
fn TagTreeNode(
    tag: Tag,
    depth: usize,
    selected_tag: ReadSignal<Option<u32>>,
    set_selected_tag: WriteSignal<Option<u32>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let id = tag.id;
    let name = tag.name.clone();
    let color = tag.color.clone().unwrap_or_else(|| "#666".to_string());
    let indent = depth * 16;
    
    // Load children and parent count
    let (children, set_children) = signal(Vec::<Tag>::new());
    let (parent_count, set_parent_count) = signal(0usize);
    let (expanded, set_expanded) = signal(true);
    
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
    Effect::new(move |_| {
        // Re-run when reload_trigger changes
        let _ = ctx.reload_trigger.get();
        spawn_local(async move {
            if let Ok(child_tags) = commands::get_tag_children(id).await {
                set_children.set(child_tags);
            }
        });
    });
    
    let is_selected = move || selected_tag.get() == Some(id);
    let has_children = move || !children.get().is_empty();
    
    // Right-click to edit
    let on_context_menu = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        set_editing_target.set(Some(EditTarget::Tag(id, name.clone())));
    };

    view! {
        <div class="tag-tree-item">
            <div
                class=move || if is_selected() { "tag-tree-row selected" } else { "tag-tree-row" }
                style=format!("padding-left: {}px;", indent + 8)
                on:click=move |_| set_selected_tag.set(Some(id))
                on:contextmenu=on_context_menu
            >
                // Expand/collapse toggle
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
                
                // Tag color dot
                <span class="tag-color-dot" style=format!("background-color: {};", color)></span>
                
                // Tag name
                <span class="tag-tree-name">{tag.name}</span>
            </div>
            
            // Children (recursive)
            {move || if expanded.get() {
                let children_list = children.get();
                view! {
                    <div class="tag-tree-children">
                        <For
                            each=move || children.get()
                            key=|child| child.id
                            children=move |child| {
                                view! {
                                    <TagTreeNode
                                        tag=child
                                        depth=depth + 1
                                        selected_tag=selected_tag
                                        set_selected_tag=set_selected_tag
                                        set_editing_target=set_editing_target
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

/// Edit target type
#[derive(Clone, Debug)]
pub enum EditTarget {
    Tag(u32, String),
    Item(u32, String),
}

/// Tag column sidebar
#[component]
pub fn TagColumn(
    selected_tag: ReadSignal<Option<u32>>,
    set_selected_tag: WriteSignal<Option<u32>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    let ctx = use_context::<AppContext>().expect("AppContext should be provided");
    
    let (root_tags, set_root_tags) = signal(Vec::<Tag>::new());
    
    // Load root tags (tags with no parents)
    Effect::new(move |_| {
        let _ = ctx.reload_trigger.get();
        spawn_local(async move {
            if let Ok(tags) = commands::get_root_tags().await {
                set_root_tags.set(tags);
            }
        });
    });

    view! {
        <div class="tag-column">
            <div class="tag-column-header">"Tags"</div>
            
            <TagAddInput />
            
            <div class="tag-tree">
                <For
                    each=move || root_tags.get()
                    key=|tag| tag.id
                    children=move |tag| {
                        view! {
                            <TagTreeNode
                                tag=tag
                                depth=0
                                selected_tag=selected_tag
                                set_selected_tag=set_selected_tag
                                set_editing_target=set_editing_target
                            />
                        }
                    }
                />
            </div>
            
            {move || if root_tags.get().is_empty() {
                view! { <div class="no-tags-message">"No tags yet"</div> }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}
