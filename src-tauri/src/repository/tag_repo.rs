//! Tag Repository Implementation
//!
//! SQLite-backed implementation for Tag CRUD and item-tag relationships.

use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::{Tag, ItemTag, DomainError, DomainResult};
use super::traits::Repository;

/// SQLite implementation of Tag repository
pub struct TagRepository {
    conn: Arc<Mutex<Connection>>,
}

impl TagRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // ========================
    // Item-Tag Relationships
    // ========================

    /// Add a tag to an item
    pub async fn add_tag_to_item(&self, item_id: u32, tag_id: u32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?, ?)",
            libsql::params![item_id, tag_id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Remove a tag from an item
    pub async fn remove_tag_from_item(&self, item_id: u32, tag_id: u32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "DELETE FROM item_tags WHERE item_id = ? AND tag_id = ?",
            libsql::params![item_id, tag_id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Get all tags for an item (sorted by pinyin for Chinese)
    pub async fn get_tags_for_item(&self, item_id: u32) -> DomainResult<Vec<Tag>> {
        use pinyin::ToPinyin;
        
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT t.id, t.name, t.color FROM tags t
                 JOIN item_tags it ON t.id = it.tag_id
                 WHERE it.item_id = ?",
                libsql::params![item_id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut tags = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            tags.push(row_to_tag(&row)?);
        }
        
        // Sort by pinyin for Chinese text
        tags.sort_by(|a, b| {
            let a_pinyin: String = a.name.chars()
                .map(|c| c.to_pinyin().map(|p| p.plain()).unwrap_or_else(|| c.to_string().leak()))
                .collect::<Vec<_>>()
                .join("");
            let b_pinyin: String = b.name.chars()
                .map(|c| c.to_pinyin().map(|p| p.plain()).unwrap_or_else(|| c.to_string().leak()))
                .collect::<Vec<_>>()
                .join("");
            a_pinyin.cmp(&b_pinyin)
        });
        
        // DEBUG: Print sorted tag names
        let tag_names: Vec<&str> = tags.iter().map(|t| t.name.as_str()).collect();
        println!("[DEBUG get_tags_for_item {}] Pinyin sorted: {:?}", item_id, tag_names);
        
        Ok(tags)
    }

    /// Get all items with a specific tag
    pub async fn get_items_with_tag(&self, tag_id: u32) -> DomainResult<Vec<u32>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT item_id FROM item_tags WHERE tag_id = ?",
                libsql::params![tag_id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut item_ids = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            item_ids.push(row.get::<u32>(0).map_err(|e| DomainError::Internal(e.to_string()))?);
        }
        Ok(item_ids)
    }

    // ========================
    // Tag-Tag Relationships (multi-parent)
    // ========================

    /// Add a parent tag to a child tag
    pub async fn add_parent_tag(&self, child_tag_id: u32, parent_tag_id: u32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        // Get next position under this parent
        let mut rows = conn
            .query(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM tag_tags WHERE parent_tag_id = ?",
                libsql::params![parent_tag_id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        let position: i32 = if let Ok(Some(row)) = rows.next().await {
            row.get::<i32>(0).unwrap_or(0)
        } else {
            0
        };
        
        conn.execute(
            "INSERT OR IGNORE INTO tag_tags (child_tag_id, parent_tag_id, position) VALUES (?, ?, ?)",
            libsql::params![child_tag_id, parent_tag_id, position],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Remove a parent tag from a child tag
    pub async fn remove_parent_tag(&self, child_tag_id: u32, parent_tag_id: u32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "DELETE FROM tag_tags WHERE child_tag_id = ? AND parent_tag_id = ?",
            libsql::params![child_tag_id, parent_tag_id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Get all parent tags for a given tag
    pub async fn get_parent_tags(&self, tag_id: u32) -> DomainResult<Vec<Tag>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT t.id, t.name, t.color FROM tags t
                 JOIN tag_tags tt ON t.id = tt.parent_tag_id
                 WHERE tt.child_tag_id = ?
                 ORDER BY t.name",
                libsql::params![tag_id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut tags = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            tags.push(row_to_tag(&row)?);
        }
        Ok(tags)
    }

    /// Get all child tags for a given parent tag (sorted by position)
    /// Returns tags with position from tag_tags table (child position under this parent)
    pub async fn get_child_tags(&self, parent_tag_id: u32) -> DomainResult<Vec<Tag>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT t.id, t.name, t.color, tt.position FROM tags t
                 JOIN tag_tags tt ON t.id = tt.child_tag_id
                 WHERE tt.parent_tag_id = ?
                 ORDER BY tt.position",
                libsql::params![parent_tag_id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut tags = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            tags.push(row_to_tag(&row)?);
        }
        Ok(tags)
    }

    /// Get root tags (tags that have no parent tags)
    pub async fn get_root_tags(&self) -> DomainResult<Vec<Tag>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT id, name, color, position FROM tags 
                 WHERE id NOT IN (SELECT DISTINCT child_tag_id FROM tag_tags)
                 ORDER BY position, name",
                (),
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut tags = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            tags.push(row_to_tag(&row)?);
        }
        Ok(tags)
    }
    
    /// Move a root tag to a new position
    pub async fn move_tag(&self, id: u32, position: i32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        // Shift existing tags at target position down
        conn.execute(
            "UPDATE tags SET position = position + 1 WHERE position >= ? AND id != ? AND id NOT IN (SELECT DISTINCT child_tag_id FROM tag_tags)",
            libsql::params![position, id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        // Update the tag's position
        conn.execute(
            "UPDATE tags SET position = ? WHERE id = ?",
            libsql::params![position, id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
    
    /// Move a child tag to a new position under a specific parent
    /// This updates the position in tag_tags table
    pub async fn move_child_tag(&self, child_tag_id: u32, parent_tag_id: u32, position: i32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        // Shift existing children at target position down
        conn.execute(
            "UPDATE tag_tags SET position = position + 1 WHERE parent_tag_id = ? AND position >= ? AND child_tag_id != ?",
            libsql::params![parent_tag_id, position, child_tag_id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        // Update the child's position under this parent
        conn.execute(
            "UPDATE tag_tags SET position = ? WHERE child_tag_id = ? AND parent_tag_id = ?",
            libsql::params![position, child_tag_id, parent_tag_id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl Repository<Tag> for TagRepository {
    async fn create(&self, entity: &Tag) -> DomainResult<Tag> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT INTO tags (name, color) VALUES (?, ?)",
            libsql::params![entity.name.clone(), entity.color.clone()],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let id = conn.last_insert_rowid() as u32;
        
        let mut tag = entity.clone();
        tag.id = id;
        Ok(tag)
    }

    async fn find_by_id(&self, id: u32) -> DomainResult<Option<Tag>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT id, name, color FROM tags WHERE id = ?",
                libsql::params![id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Ok(Some(row)) = rows.next().await {
            Ok(Some(row_to_tag(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn list(&self) -> DomainResult<Vec<Tag>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query("SELECT id, name, color FROM tags ORDER BY name", ())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut tags = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            tags.push(row_to_tag(&row)?);
        }
        Ok(tags)
    }

    async fn update(&self, entity: &Tag) -> DomainResult<Tag> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE tags SET name = ?, color = ? WHERE id = ?",
            libsql::params![entity.name.clone(), entity.color.clone(), entity.id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(entity.clone())
    }

    async fn delete(&self, id: u32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        // CASCADE will remove item_tags entries
        conn.execute("DELETE FROM tags WHERE id = ?", libsql::params![id])
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

/// Convert a database row to Tag
fn row_to_tag(row: &libsql::Row) -> DomainResult<Tag> {
    Ok(Tag {
        id: row.get::<u32>(0).map_err(|e| DomainError::Internal(e.to_string()))?,
        name: row.get::<String>(1).map_err(|e| DomainError::Internal(e.to_string()))?,
        color: row.get::<Option<String>>(2).ok().flatten(),
        position: row.get::<i32>(3).unwrap_or(0),
    })
}
