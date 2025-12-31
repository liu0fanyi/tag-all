//! Item Hierarchy Operations
//!
//! Operations for managing parent-child relationships between items.

use async_trait::async_trait;

use crate::domain::{Item, DomainError, DomainResult};

/// Trait for item hierarchy operations
#[async_trait]
pub trait ItemHierarchyOperations {
    /// Get children of a parent item
    async fn get_children(&self, parent_id: Option<u32>) -> DomainResult<Vec<Item>>;
    
    /// Move item to a new parent and position
    async fn move_to(&self, id: u32, new_parent_id: Option<u32>, position: i32) -> DomainResult<()>;
    
    /// Get all descendants of an item recursively
    async fn get_descendants(&self, id: u32) -> DomainResult<Vec<Item>>;
    
    /// Toggle collapsed state of an item
    async fn toggle_collapsed(&self, id: u32) -> DomainResult<bool>;
}

#[async_trait]
impl ItemHierarchyOperations for super::item_repo::ItemRepository {
    async fn get_children(&self, parent_id: Option<u32>) -> DomainResult<Vec<Item>> {
        let conn = self.conn.lock().await;
        
        let mut rows = match parent_id {
            Some(pid) => conn.query(
                "SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed FROM items WHERE parent_id = ? ORDER BY position",
                libsql::params![pid],
            ).await.map_err(|e| DomainError::Internal(e.to_string()))?,
            None => conn.query(
                "SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed FROM items WHERE parent_id IS NULL ORDER BY position",
                (),
            ).await.map_err(|e| DomainError::Internal(e.to_string()))?,
        };
        
        let mut items = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            items.push(super::item_repo::row_to_item(&row)?);
        }
        Ok(items)
    }

    async fn move_to(&self, id: u32, new_parent_id: Option<u32>, position: i32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        // Shift existing items at target position down
        match new_parent_id {
            Some(pid) => {
                conn.execute(
                    "UPDATE items SET position = position + 1 WHERE parent_id = ? AND position >= ? AND id != ?",
                    libsql::params![pid, position, id],
                )
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            }
            None => {
                conn.execute(
                    "UPDATE items SET position = position + 1 WHERE parent_id IS NULL AND position >= ? AND id != ?",
                    libsql::params![position, id],
                )
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            }
        }
        
        // Move the item
        conn.execute(
            "UPDATE items SET parent_id = ?, position = ? WHERE id = ?",
            libsql::params![new_parent_id, position, id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        // Drop conn and reindex items under the parent
        drop(conn);
        
        use super::item_positioning::ItemPositioningOperations;
        self.reindex_items(new_parent_id).await?;

        Ok(())
    }

    async fn get_descendants(&self, id: u32) -> DomainResult<Vec<Item>> {
        let conn = self.conn.lock().await;
        let mut result = Vec::new();
        let mut to_visit = vec![id];
        
        while let Some(current_id) = to_visit.pop() {
            let mut rows = conn.query(
                "SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed FROM items WHERE parent_id = ?",
                libsql::params![current_id],
            ).await.map_err(|e| DomainError::Internal(e.to_string()))?;
            
            while let Ok(Some(row)) = rows.next().await {
                let item = super::item_repo::row_to_item(&row)?;
                to_visit.push(item.id);
                result.push(item);
            }
        }
        
        Ok(result)
    }

    async fn toggle_collapsed(&self, id: u32) -> DomainResult<bool> {
        let conn = self.conn.lock().await;
        
        // Get current collapsed state
        let mut rows = conn.query(
            "SELECT collapsed FROM items WHERE id = ?",
            libsql::params![id],
        ).await.map_err(|e| DomainError::Internal(e.to_string()))?;
        
        let current = if let Ok(Some(row)) = rows.next().await {
            row.get::<bool>(0).unwrap_or(false)
        } else {
            return Err(DomainError::NotFound(format!("Item {} not found", id)));
        };
        drop(rows);
        
        // Toggle it
        let new_state = !current;
        conn.execute(
            "UPDATE items SET collapsed = ? WHERE id = ?",
            libsql::params![new_state, id],
        ).await.map_err(|e| DomainError::Internal(e.to_string()))?;
        
        Ok(new_state)
    }
}
