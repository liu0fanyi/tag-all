//! Item Entity
//!
//! Represents a todo/task item. At Level 1, this is a simple flat structure.
//! Parent-child relationships will be added in Level 2.

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

/// A todo/task item
///
/// Level 1: Basic flat structure
/// Level 2 will add: parent_id, position
/// Level 3 will add: tag_ids
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
    // Level 2 fields (commented for now):
    // pub parent_id: Option<u32>,
    // pub position: i32,
    // pub collapsed: bool,
    // Level 5 field:
    // pub workspace_id: u32,
}

impl Item {
    /// Create a new item with default values
    pub fn new(id: u32, text: String, item_type: ItemType) -> Self {
        Self {
            id,
            text,
            completed: false,
            item_type,
            memo: None,
            target_count: None,
            current_count: 0,
        }
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
    }

    #[test]
    fn test_item_type_serialization() {
        assert_eq!(ItemType::Daily.as_str(), "daily");
        assert_eq!(ItemType::from_str("countdown"), ItemType::Countdown);
    }
}
