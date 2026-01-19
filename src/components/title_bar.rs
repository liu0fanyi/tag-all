//! Custom Title Bar Component
//!
//! Provides window controls (minimize, close, pin) in a draggable title bar.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::commands;

/// Custom title bar with window controls
#[component]
pub fn TitleBar(
    is_pinned: ReadSignal<bool>,
    set_is_pinned: WriteSignal<bool>,
    // Sync parameters
    sync_url: ReadSignal<String>,
    sync_token: ReadSignal<String>,
    sync_status: ReadSignal<String>,
    sync_msg: ReadSignal<String>,
    on_sync_click: Callback<()>,
    on_sync_right_click: Callback<()>,
) -> impl IntoView {
    // Toggle pin
    let toggle_pin = move |_| {
        let new_pinned = !is_pinned.get();
        set_is_pinned.set(new_pinned);
        spawn_local(async move {
            let _ = commands::set_pinned(new_pinned).await;
        });
    };
    
    // Minimize window
    let minimize = move |_| {
        spawn_local(async {
            let _ = commands::minimize_window().await;
        });
    };
    
    // Close window
    let close = move |_| {
        spawn_local(async {
            let _ = commands::close_window().await;
        });
    };
    
    // Dynamic class for titlebar based on pinned state
    let titlebar_class = move || {
        if is_pinned.get() {
            "custom-titlebar pinned"
        } else {
            "custom-titlebar"
        }
    };

    view! {
        <div class=titlebar_class>
            // Drag region only when NOT pinned
            <Show when=move || !is_pinned.get()>
                <div class="titlebar-drag-region" data-tauri-drag-region>
                    <img src="public/icon.png" class="titlebar-icon" alt="" />
                    <span class="titlebar-title">"Tag-All"</span>
                </div>
            </Show>
            <Show when=move || is_pinned.get()>
                <div class="titlebar-drag-region locked">
                    <img src="public/icon.png" class="titlebar-icon" alt="" />
                    <span class="titlebar-title">"Tag-All"</span>
                    <span class="lock-icon">"🔒"</span>
                </div>
            </Show>
            
            <div class="titlebar-controls">
                <button
                    class="titlebar-btn sync"
                    title=move || {
                        let has_config = !sync_url.get().is_empty() && !sync_token.get().is_empty();
                        let msg = sync_msg.get();
                        if has_config {
                            if !msg.is_empty() {
                                format!("{}\n左键：配置同步\n右键：立即同步", msg)
                            } else {
                                "左键：配置同步\n右键：立即同步".to_string()
                            }
                        } else {
                            "左键：配置同步".to_string()
                        }
                    }
                    on:click=move |ev| {
                        ev.stop_propagation();
                        on_sync_click.run(());
                    }
                    on:contextmenu=move |ev: web_sys::MouseEvent| {
                        ev.prevent_default();
                        ev.stop_propagation();
                        on_sync_right_click.run(());
                    }
                >
                    {move || {
                        let has_config = !sync_url.get().is_empty() && !sync_token.get().is_empty();
                        let status = sync_status.get();
                        let (icon, class) = if !has_config {
                            ("⚙️", "")
                        } else {
                            match status.as_str() {
                                "syncing" | "testing" => ("🔄", "animate-spin"),
                                "success" => ("✅", ""),
                                "error" => ("❌", ""),
                                _ => ("☁️", ""),
                            }
                        };
                        view! {
                            <span class=class>{icon}</span>
                        }
                    }}
                </button>
                <button
                    class=move || if is_pinned.get() { "titlebar-btn pin active" } else { "titlebar-btn pin" }
                    title=move || if is_pinned.get() { "取消固定" } else { "固定窗口" }
                    on:click=toggle_pin
                >
                    "📌"
                </button>
                <button class="titlebar-btn minimize" title="最小化" on:click=minimize>
                    "─"
                </button>
                <button class="titlebar-btn close" title="关闭" on:click=close>
                    "✕"
                </button>
            </div>
        </div>
    }
}
