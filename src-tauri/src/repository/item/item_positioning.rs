//! Item Positioning Operations
//!
//! Operations for managing item positions within their parent hierarchy.

use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::{DomainError, DomainResult};

/// Trait for item positioning operations
#[async_trait]
pub trait ItemPositioningOperations {
    /// Get next position for a parent (used in create)
    async fn get_next_position(&self, parent_id: Option<u32>) -> DomainResult<i32>;
    
    /// Reindex items under a parent to be sequential (0, 1, 2, ...)
    async fn reindex_items(&self, parent_id: Option<u32>) -> DomainResult<()>;
}

#[async_trait]
impl ItemPositioningOperations for super::item_repo::ItemRepository {
    async fn get_next_position(&self, parent_id: Option<u32>) -> DomainResult<i32> {
        let conn = self.conn.lock().await;
        
        let query = match parent_id {
            Some(pid) => format!(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM items WHERE parent_id = {}", pid
            ),
            None => "SELECT COALESCE(MAX(position), -1) + 1 FROM items WHERE parent_id IS NULL".to_string(),
        };
        
        let mut rows = conn.query(&query, ())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        if let Ok(Some(row)) = rows.next().await {
            Ok(row.get::<i32>(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn reindex_items(&self, parent_id: Option<u32>) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        // Get all items under this parent ordered by current position
        let mut rows = match parent_id {
            Some(pid) => conn
                .query(
                    "SELECT id FROM items WHERE parent_id = ? ORDER BY position, id",
                    libsql::params![pid],
                )
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?,
            None => conn
                .query(
                    "SELECT id FROM items WHERE parent_id IS NULL ORDER BY position, id",
                    (),
                )
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?,
        };
        
        let mut ids = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            let id: u32 = row.get(0).map_err(|e| DomainError::Internal(e.to_string()))?;
            ids.push(id);
        }
        drop(rows);
        
        // Update each item with sequential position
        for (new_pos, id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE items SET position = ? WHERE id = ?",
                libsql::params![new_pos as i32, *id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        }
        
        Ok(())
    }
}
