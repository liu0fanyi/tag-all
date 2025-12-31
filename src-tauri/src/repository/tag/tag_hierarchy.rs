//! Tag Hierarchy Operations
//!
//! Operations for managing parent-child relationships between tags (tag_tags table).

use async_trait::async_trait;

use crate::domain::{Tag, DomainError, DomainResult};

/// Trait for tag hierarchy operations
#[async_trait]
pub trait TagHierarchyOperations {
    /// Add a parent tag to a child tag
    async fn add_parent_tag(&self, child_tag_id: u32, parent_tag_id: u32) -> DomainResult<()>;
    
    /// Remove a parent tag from a child tag
    async fn remove_parent_tag(&self, child_tag_id: u32, parent_tag_id: u32) -> DomainResult<()>;
    
    /// Get all parent tags for a given tag
    async fn get_parent_tags(&self, tag_id: u32) -> DomainResult<Vec<Tag>>;
    
    /// Get all child tags for a given parent tag (sorted by position)
    async fn get_child_tags(&self, parent_tag_id: u32) -> DomainResult<Vec<Tag>>;
    
    /// Get root tags (tags that have no parent tags)
    async fn get_root_tags(&self) -> DomainResult<Vec<Tag>>;
}

#[async_trait]
impl TagHierarchyOperations for super::tag_repo::TagRepository {
    async fn add_parent_tag(&self, child_tag_id: u32, parent_tag_id: u32) -> DomainResult<()> {
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
        
        // Drop conn and reindex root tags since a tag was removed from root
        drop(conn);
        
        // Need to call reindex_root_tags which is in tag_positioning
        // This creates a circular dependency - we'll call it via self
        use super::tag_positioning::TagPositioningOperations;
        self.reindex_root_tags().await?;

        Ok(())
    }

    async fn remove_parent_tag(&self, child_tag_id: u32, parent_tag_id: u32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "DELETE FROM tag_tags WHERE child_tag_id = ? AND parent_tag_id = ?",
            libsql::params![child_tag_id, parent_tag_id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        // Drop conn and reindex root tags since a tag was added back to root
        drop(conn);
        
        use super::tag_positioning::TagPositioningOperations;
        self.reindex_root_tags().await?;

        Ok(())
    }

    async fn get_parent_tags(&self, tag_id: u32) -> DomainResult<Vec<Tag>> {
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
            tags.push(super::tag_repo::row_to_tag(&row)?);
        }
        Ok(tags)
    }

    async fn get_child_tags(&self, parent_tag_id: u32) -> DomainResult<Vec<Tag>> {
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
            tags.push(super::tag_repo::row_to_tag(&row)?);
        }
        Ok(tags)
    }

    async fn get_root_tags(&self) -> DomainResult<Vec<Tag>> {
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
            tags.push(super::tag_repo::row_to_tag(&row)?);
        }
        
        Ok(tags)
    }
}
