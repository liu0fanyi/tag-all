//! Global Application State Store
//!
//! Uses Leptos reactive_stores for fine-grained reactivity.

use leptos::prelude::*;
use reactive_stores::Store;
use crate::models::{Item, Tag, Workspace};

/// Global application state with field-level reactivity
#[derive(Clone, Debug, Default, Store)]
pub struct AppState {
    /// All items in current workspace
    pub items: Vec<Item>,
    /// All tags
    pub tags: Vec<Tag>,
    /// Root tags (tags with no parent) for TagColumn
    pub root_tags: Vec<Tag>,
    /// All workspaces
    pub workspaces: Vec<Workspace>,
    /// Current workspace ID
    pub current_workspace_id: u32,
    /// Version counter for tag relation changes (item-tag and tag-tag, increment to trigger reload)
    pub tags_relation_version: u32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_workspace_id: 1,
            ..Default::default()
        }
    }
}

/// Type alias for the store
pub type AppStore = Store<AppState>;

/// Get the app store from context
pub fn use_app_store() -> AppStore {
    expect_context::<AppStore>()
}

// ========================
// Store Helper Functions
// ========================

/// Update an item in the store by ID
pub fn store_update_item(store: &AppStore, updated_item: Item) {
    store.items().write().iter_mut()
        .find(|item| item.id == updated_item.id)
        .map(|item| *item = updated_item);
}

/// Remove an item from the store by ID
pub fn store_remove_item(store: &AppStore, item_id: u32) {
    store.items().write().retain(|item| item.id != item_id);
}

/// Add a tag to the store
pub fn store_add_tag(store: &AppStore, tag: Tag) {
    store.tags().write().push(tag);
}

/// Update a tag in the store by ID
pub fn store_update_tag(store: &AppStore, updated_tag: Tag) {
    store.tags().write().iter_mut()
        .find(|tag| tag.id == updated_tag.id)
        .map(|tag| *tag = updated_tag);
}

/// Remove a tag from the store by ID (from both tags and root_tags)
pub fn store_remove_tag(store: &AppStore, tag_id: u32) {
    store.tags().write().retain(|tag| tag.id != tag_id);
    store.root_tags().write().retain(|tag| tag.id != tag_id);
}

/// Add a workspace to the store
pub fn store_add_workspace(store: &AppStore, workspace: Workspace) {
    store.workspaces().write().push(workspace);
}
