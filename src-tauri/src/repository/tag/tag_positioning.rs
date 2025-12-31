//! Tag Positioning Operations
//!
//! Operations for managing tag positions in both tags table and tag_tags table.

use async_trait::async_trait;

use crate::domain::{DomainError, DomainResult};

/// Trait for tag positioning operations
#[async_trait]
pub trait TagPositioningOperations {
    /// Move a root tag to a new position
    async fn move_tag(&self, id: u32, new_position: i32) -> DomainResult<()>;
    
    /// Reindex all root tag positions to be sequential (0, 1, 2, ...)
    async fn reindex_root_tags(&self) -> DomainResult<()>;
    
    /// Move a child tag to a new position under a specific parent
    async fn move_child_tag(&self, child_tag_id: u32, parent_tag_id: u32, position: i32) -> DomainResult<()>;
}

#[async_trait]
impl TagPositioningOperations for super::tag_repo::TagRepository {
    async fn move_tag(&self, id: u32, new_position: i32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        // Get old position
        let mut rows = conn
            .query("SELECT position FROM tags WHERE id = ?", libsql::params![id])
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        let old_position: i32 = if let Ok(Some(row)) = rows.next().await {
            row.get::<i32>(0).unwrap_or(0)
        } else {
            return Err(DomainError::NotFound(format!("Tag {} not found", id)));
        };
        
        if old_position == new_position {
            return Ok(());
        }
        
        if new_position < old_position {
            // Moving up: shift tags in [new_position, old_position) down by +1
            conn.execute(
                "UPDATE tags SET position = position + 1 WHERE position >= ? AND position < ? AND id NOT IN (SELECT DISTINCT child_tag_id FROM tag_tags)",
                libsql::params![new_position, old_position],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        } else {
            // Moving down: shift tags in (old_position, new_position] up by -1
            conn.execute(
                "UPDATE tags SET position = position - 1 WHERE position > ? AND position <= ? AND id NOT IN (SELECT DISTINCT child_tag_id FROM tag_tags)",
                libsql::params![old_position, new_position],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        }
        
        // Update the tag's position
        conn.execute(
            "UPDATE tags SET position = ? WHERE id = ?",
            libsql::params![new_position, id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        // Drop conn before calling reindex (which also needs conn)
        drop(conn);
        
        // Reindex all root tag positions to ensure no gaps or duplicates
        self.reindex_root_tags().await?;

        Ok(())
    }
    
    async fn reindex_root_tags(&self) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        // Get all root tags ordered by current position
        let mut rows = conn
            .query(
                "SELECT id FROM tags 
                 WHERE id NOT IN (SELECT DISTINCT child_tag_id FROM tag_tags)
                 ORDER BY position, id",
                (),
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        let mut ids = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            let id: u32 = row.get(0).map_err(|e| DomainError::Internal(e.to_string()))?;
            ids.push(id);
        }
        drop(rows);
        
        // Update each tag with sequential position
        for (new_pos, id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE tags SET position = ? WHERE id = ?",
                libsql::params![new_pos as i32, *id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        }
        
        Ok(())
    }
    
    async fn move_child_tag(&self, child_tag_id: u32, parent_tag_id: u32, position: i32) -> DomainResult<()> {
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
