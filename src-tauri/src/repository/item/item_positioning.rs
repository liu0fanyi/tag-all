//! Item Positioning Operations
//!
//! Operations for managing item positions within their parent hierarchy.

use async_trait::async_trait;
use rusqlite::params;

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
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let query = match parent_id {
            Some(pid) => format!(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM items WHERE parent_id = {}", pid
            ),
            None => "SELECT COALESCE(MAX(position), -1) + 1 FROM items WHERE parent_id IS NULL".to_string(),
        };
        
        let mut stmt = conn.prepare(&query)
             .map_err(|e| DomainError::Internal(e.to_string()))?;
             
        let mut rows = stmt.query([])
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        if let Ok(Some(row)) = rows.next() {
            Ok(row.get::<_, i32>(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn reindex_items(&self, parent_id: Option<u32>) -> DomainResult<()> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        // Get all items under this parent ordered by current position
        let mut ids = Vec::new();
        
        {
            let mut stmt = match parent_id {
                Some(pid) => conn.prepare("SELECT id FROM items WHERE parent_id = ? ORDER BY position, id").map_err(|e| DomainError::Internal(e.to_string()))?,
                None => conn.prepare("SELECT id FROM items WHERE parent_id IS NULL ORDER BY position, id").map_err(|e| DomainError::Internal(e.to_string()))?,
            };
            
            let mut rows = match parent_id {
                Some(pid) => stmt.query(params![pid]).map_err(|e| DomainError::Internal(e.to_string()))?,
                None => stmt.query([]).map_err(|e| DomainError::Internal(e.to_string()))?,
            };
            
            while let Ok(Some(row)) = rows.next() {
                let id: u32 = row.get(0).map_err(|e| DomainError::Internal(e.to_string()))?;
                ids.push(id);
            }
        }
        
        // Update each item with sequential position
        for (new_pos, id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE items SET position = ?, updated_at = ? WHERE id = ?",
                params![new_pos as i32, chrono::Utc::now().timestamp_millis(), *id],
            )
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        }
        
        Ok(())
    }
}
