//! Application Context
//!
//! Shared state provided via Leptos Context API.

use leptos::prelude::*;

/// App-wide signals provided via context
#[derive(Clone, Copy)]
pub struct AppContext {
    /// Trigger to reload items from backend - read
    pub reload_trigger: ReadSignal<u32>,
    /// Trigger to reload items from backend - write
    set_reload_trigger: WriteSignal<u32>,
    /// Which item to add a child under (None = root) - read
    pub adding_under: ReadSignal<Option<u32>>,
    /// Which item to add a child under (None = root) - write
    set_adding_under: WriteSignal<Option<u32>>,
    /// Current workspace ID - read
    pub current_workspace: ReadSignal<u32>,
}

impl AppContext {
    pub fn new(
        reload_trigger: (ReadSignal<u32>, WriteSignal<u32>),
        adding_under: (ReadSignal<Option<u32>>, WriteSignal<Option<u32>>),
        current_workspace: ReadSignal<u32>,
    ) -> Self {
        Self {
            reload_trigger: reload_trigger.0,
            set_reload_trigger: reload_trigger.1,
            adding_under: adding_under.0,
            set_adding_under: adding_under.1,
            current_workspace,
        }
    }
    
    /// Trigger a reload of items
    pub fn reload(&self) {
        self.set_reload_trigger.update(|v| *v += 1);
    }
    
    /// Set parent for new child item
    pub fn set_adding_under(&self, parent_id: Option<u32>) {
        self.set_adding_under.set(parent_id);
    }
}
