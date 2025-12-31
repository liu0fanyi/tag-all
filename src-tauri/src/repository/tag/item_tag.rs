//! Item-Tag Relationship Operations
//!
//! Operations for managing the many-to-many relationship between items and tags.

use async_trait::async_trait;

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
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?, ?)",
            libsql::params![item_id, tag_id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn remove_tag_from_item(&self, item_id: u32, tag_id: u32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "DELETE FROM item_tags WHERE item_id = ? AND tag_id = ?",
            libsql::params![item_id, tag_id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn get_tags_for_item(&self, item_id: u32) -> DomainResult<Vec<Tag>> {
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
}
