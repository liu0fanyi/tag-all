use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::commands;
use crate::models::{FileViewItem, Item};
use crate::components::TagDndContext;
use leptos_dragdrop::{make_on_mouseleave, make_on_file_mouseenter, DropTarget};

#[component]
pub fn FileList(
    path: Signal<Option<String>>,
    // We might need to communicate selection back to parent/global context
    set_selected_file: WriteSignal<Option<FileViewItem>>,
) -> impl IntoView {
    let tag_dnd = use_context::<TagDndContext>().expect("TagDndContext should be provided");
    let dnd = tag_dnd.dnd;

    let (files, set_files) = signal(Vec::<FileViewItem>::new());
    let (loading, set_loading) = signal(false);

    Effect::new(move |_| {
        let current_path = path.get();
        web_sys::console::log_1(&format!("[FileList] Path changed to: {:?}", current_path).into());
        
        if let Some(p) = current_path {
             set_loading.set(true);
             spawn_local(async move {
                 match commands::list_directory(&p).await {
                     Ok(res) => {
                         web_sys::console::log_1(&format!("[FileList] Loaded {} files from {}", res.len(), p).into());
                         set_files.set(res)
                     },
                     Err(e) => {
                         web_sys::console::error_1(&format!("[FileList] Error loading {}: {}", p, e).into());
                     }
                 }
                 set_loading.set(false);
             });
        } else {
            set_files.set(Vec::new());
        }
    });

    // We also need to listen to global reload trigger to refresh list if a tag is added
    let ctx = use_context::<crate::context::AppContext>().expect("AppContext should be provided");
    let reload_trigger = ctx.reload_trigger;
    
    Effect::new(move |_| {
        let _ = reload_trigger.get();
        let current_path = path.get();
        if let Some(p) = current_path {
             spawn_local(async move {
                 if let Ok(res) = commands::list_directory(&p).await {
                     set_files.set(res);
                 }
             });
        }
    });
    
    // Icon helper
    let get_icon = |is_dir: bool| {
        if is_dir { "📁" } else { "📄" }
    };
    
    // Format size
    let format_size = |size: u64| {
        if size < 1024 { format!("{} B", size) }
        else if size < 1024 * 1024 { format!("{:.1} KB", size as f64 / 1024.0) }
        else { format!("{:.1} MB", size as f64 / (1024.0 * 1024.0)) }
    };

    view! {
        <div class="file-list-container">
            <Show when=move || loading.get()>
                <div class="loading">"Loading..."</div>
            </Show>
            
            <div class="file-grid">
                <For
                    each=move || files.get()
                    key=|item| item.path.clone()
                    children=move |item| {
                        let item_clone = item.clone();
                        let is_tagged = item.db_item.is_some();
                        let icon = get_icon(item.is_dir);
                        let name = item.name.clone();
                        let size = item.size;
                        
                        // DnD Handlers
                        let item_path = item.path.clone();
                        let on_mouseenter = make_on_file_mouseenter(dnd, item_path.clone());
                        let on_mouseleave = make_on_mouseleave(dnd);
                        
                        let is_drop_target = move || {
                            matches!(dnd.drop_target_read.get(), Some(DropTarget::File(ref p)) if *p == item_path)
                        };
                        
                        view! {
                            <div 
                                class="file-card"
                                class:tagged=is_tagged
                                class:drop-target=is_drop_target
                                on:click=move |_| set_selected_file.set(Some(item_clone.clone()))
                                on:mouseenter=on_mouseenter
                                on:mouseleave=on_mouseleave
                            >
                                <div class="file-icon">{icon}</div>
                                <div class="file-name" title={name.clone()}>{name.clone()}</div>
                                <div class="file-meta">
                                    <span class="file-size">{if !item.is_dir { format_size(size) } else { "".to_string() }}</span>
                                    {if is_tagged {
                                        view! { <span class="tag-indicator">"🏷️"</span> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
