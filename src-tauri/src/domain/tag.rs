//! Tag Entity
//!
//! Tags can be attached to items for categorization and filtering.

use serde::{Deserialize, Serialize};
use super::entity::Entity;

/// A tag for categorizing items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    /// Unique identifier
    pub id: u32,
    /// Tag name
    pub name: String,
    /// Color (hex, e.g., "#FF5733")
    pub color: Option<String>,
}

impl Tag {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            color: None,
        }
    }

    pub fn with_color(id: u32, name: String, color: String) -> Self {
        Self {
            id,
            name,
            color: Some(color),
        }
    }
}

impl Entity for Tag {
    type Id = u32;

    fn id(&self) -> Self::Id {
        self.id
    }
}

/// Join table entry for item-tag relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTag {
    pub item_id: u32,
    pub tag_id: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new(1, "Work".to_string());
        assert_eq!(tag.id(), 1);
        assert_eq!(tag.name, "Work");
        assert!(tag.color.is_none());
    }

    #[test]
    fn test_tag_with_color() {
        let tag = Tag::with_color(2, "Urgent".to_string(), "#FF0000".to_string());
        assert_eq!(tag.color, Some("#FF0000".to_string()));
    }
}
