//! Item Entity
//!
//! Represents a todo/task item with hierarchical structure (single parent).

use serde::{Deserialize, Serialize};
use super::entity::Entity;

/// Item type determines behavior and appearance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    /// Resets daily
    #[default]
    Daily,
    /// One-time task, deleted when completed
    Once,
    /// Countdown task with target count
    Countdown,
    /// Document/note (no checkbox)
    Document,
    /// Label/tag type (will be separate Tag entity in Level 3)
    Label,
}

impl ItemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemType::Daily => "daily",
            ItemType::Once => "once",
            ItemType::Countdown => "countdown",
            ItemType::Document => "document",
            ItemType::Label => "label",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "once" => ItemType::Once,
            "countdown" => ItemType::Countdown,
            "document" => ItemType::Document,
            "label" => ItemType::Label,
            _ => ItemType::Daily,
        }
    }
}

/// A todo/task item with hierarchical structure
///
/// Level 2: Added parent_id, position, collapsed for tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Unique identifier
    pub id: u32,
    /// Item text content
    pub text: String,
    /// Completion status
    pub completed: bool,
    /// Item type
    pub item_type: ItemType,
    /// Optional memo/notes (Markdown content)
    pub memo: Option<String>,
    /// Target count for countdown type
    pub target_count: Option<i32>,
    /// Current count for countdown type
    pub current_count: i32,
    
    // Level 2 fields:
    /// Parent item ID (None = root level)
    pub parent_id: Option<u32>,
    /// Position within siblings (for ordering)
    pub position: i32,
    /// Whether children are collapsed in UI
    pub collapsed: bool,
    
    // Level 5 field:
    // pub workspace_id: u32,
    
    // Level 6 fields:
    pub url: Option<String>,
    pub summary: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl Item {
    /// Create a new root item with default values
    pub fn new(id: u32, text: String, item_type: ItemType) -> Self {
        Self {
            id,
            text,
            completed: false,
            item_type,
            memo: None,
            target_count: None,
            current_count: 0,
            parent_id: None,
            position: 0,
            collapsed: false,
            url: None,
            summary: None,
            created_at: None,
            updated_at: None,
        }
    }

    /// Create a new child item under a parent
    pub fn new_child(id: u32, text: String, item_type: ItemType, parent_id: u32, position: i32) -> Self {
        Self {
            id,
            text,
            completed: false,
            item_type,
            memo: None,
            target_count: None,
            current_count: 0,
            parent_id: Some(parent_id),
            position,
            collapsed: false,
            url: None,
            summary: None,
            created_at: None,
            updated_at: None,
        }
    }

    /// Check if this is a root item (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
}

impl Entity for Item {
    type Id = u32;

    fn id(&self) -> Self::Id {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_creation() {
        let item = Item::new(1, "Test item".to_string(), ItemType::Daily);
        assert_eq!(item.id(), 1);
        assert_eq!(item.text, "Test item");
        assert!(!item.completed);
        assert!(item.is_root());
    }

    #[test]
    fn test_child_item_creation() {
        let child = Item::new_child(2, "Child".to_string(), ItemType::Daily, 1, 0);
        assert_eq!(child.parent_id, Some(1));
        assert!(!child.is_root());
    }

    #[test]
    fn test_item_type_serialization() {
        assert_eq!(ItemType::Daily.as_str(), "daily");
        assert_eq!(ItemType::from_str("countdown"), ItemType::Countdown);
    }
}
