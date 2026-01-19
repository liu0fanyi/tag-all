//! Item-Tag Relationship Operations
//!
//! Operations for managing the many-to-many relationship between items and tags.

use async_trait::async_trait;
use rusqlite::params;

use crate::domain::{Tag, DomainError, DomainResult};

/// Trait for item-tag relationship operations
#[async_trait]
pub trait ItemTagOperations {
    /// Add a tag to an item
    async fn add_tag_to_item(&self, item_id: u32, tag_id: u32) -> DomainResult<()>;
    
    /// Remove a tag from an item
    async fn remove_tag_from_item(&self, item_id: u32, tag_id: u32) -> DomainResult<()>;
    
    /// Get all tags for an item (sorted by pinyin for Chinese)
    async fn get_tags_for_item(&self, item_id: u32) -> DomainResult<Vec<Tag>>;
    
    /// Get all items with a specific tag
    async fn get_items_with_tag(&self, tag_id: u32) -> DomainResult<Vec<u32>>;
}

#[async_trait]
impl ItemTagOperations for super::tag_repo::TagRepository {
    async fn add_tag_to_item(&self, item_id: u32, tag_id: u32) -> DomainResult<()> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        conn.execute(
            "INSERT OR IGNORE INTO item_tags (item_id, tag_id, updated_at) VALUES (?, ?, ?)",
            params![item_id, tag_id, chrono::Utc::now().timestamp_millis()],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn remove_tag_from_item(&self, item_id: u32, tag_id: u32) -> DomainResult<()> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        conn.execute(
            "DELETE FROM item_tags WHERE item_id = ? AND tag_id = ?",
            params![item_id, tag_id],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn get_tags_for_item(&self, item_id: u32) -> DomainResult<Vec<Tag>> {
        use pinyin::ToPinyin;
        
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = conn.prepare(
                "SELECT t.id, t.name, t.color FROM tags t
                 JOIN item_tags it ON t.id = it.tag_id
                 WHERE it.item_id = ?"
            )
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        let mut rows = stmt.query(params![item_id])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut tags = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            tags.push(super::tag_repo::row_to_tag(&row)?);
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
        
        Ok(tags)
    }

    async fn get_items_with_tag(&self, tag_id: u32) -> DomainResult<Vec<u32>> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = conn.prepare("SELECT item_id FROM item_tags WHERE tag_id = ?")
            .map_err(|e| DomainError::Internal(e.to_string()))?;
            
        let mut rows = stmt.query(params![tag_id])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut item_ids = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            item_ids.push(row.get::<_, u32>(0).map_err(|e| DomainError::Internal(e.to_string()))?);
        }
        Ok(item_ids)
    }
}
