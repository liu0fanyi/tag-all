//! Item Hierarchy Operations
//!
//! Operations for managing parent-child relationships between items.

use async_trait::async_trait;
use rusqlite::params;

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
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = match parent_id {
            Some(_) => conn.prepare("SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, url, summary, CAST(created_at AS INTEGER) as created_at, CAST(updated_at AS INTEGER) as updated_at, content_hash, quick_hash, last_known_path, is_dir FROM items WHERE parent_id = ? ORDER BY position").map_err(|e| DomainError::Internal(e.to_string()))?,
            None => conn.prepare("SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, url, summary, CAST(created_at AS INTEGER) as created_at, CAST(updated_at AS INTEGER) as updated_at, content_hash, quick_hash, last_known_path, is_dir FROM items WHERE parent_id IS NULL ORDER BY position").map_err(|e| DomainError::Internal(e.to_string()))?,
        };
        
        let mut rows = match parent_id {
            Some(pid) => stmt.query(params![pid]).map_err(|e| DomainError::Internal(e.to_string()))?,
            None => stmt.query([]).map_err(|e| DomainError::Internal(e.to_string()))?,
        };
        
        let mut items = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            items.push(super::item_repo::row_to_item(&row)?);
        }
        Ok(items)
    }

    async fn move_to(&self, id: u32, new_parent_id: Option<u32>, position: i32) -> DomainResult<()> {
        let guard = self.conn.lock().await;
        
        {
             let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
             
             // Shift existing items at target position down
             match new_parent_id {
                 Some(pid) => {
                     conn.execute(
                         "UPDATE items SET position = position + 1 WHERE parent_id = ? AND position >= ? AND id != ?",
                         params![pid, position, id],
                     )
                     .map_err(|e| DomainError::Internal(e.to_string()))?;
                 }
                 None => {
                     conn.execute(
                         "UPDATE items SET position = position + 1 WHERE parent_id IS NULL AND position >= ? AND id != ?",
                         params![position, id],
                     )
                     .map_err(|e| DomainError::Internal(e.to_string()))?;
                 }
             }
             
             // Move the item
             conn.execute(
                 "UPDATE items SET parent_id = ?, position = ?, updated_at = ? WHERE id = ?",
                 params![new_parent_id, position, chrono::Utc::now().timestamp_millis(), id],
             )
             .map_err(|e| DomainError::Internal(e.to_string()))?;
        }
        
        // Drop guard and reindex items under the parent
        drop(guard);
        
        use super::item_positioning::ItemPositioningOperations;
        self.reindex_items(new_parent_id).await?;

        Ok(())
    }

    async fn get_descendants(&self, id: u32) -> DomainResult<Vec<Item>> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut result = Vec::new();
        let mut to_visit = vec![id];
        
        let mut stmt = conn.prepare("SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, url, summary, CAST(created_at AS INTEGER) as created_at, CAST(updated_at AS INTEGER) as updated_at, content_hash, quick_hash, last_known_path, is_dir FROM items WHERE parent_id = ?")
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        while let Some(current_id) = to_visit.pop() {
            let mut rows = stmt.query(params![current_id])
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            
            while let Ok(Some(row)) = rows.next() {
                let item = super::item_repo::row_to_item(&row)?;
                to_visit.push(item.id);
                result.push(item);
            }
        }
        
        Ok(result)
    }

    async fn toggle_collapsed(&self, id: u32) -> DomainResult<bool> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        // Get current collapsed state
        let mut stmt = conn.prepare("SELECT collapsed FROM items WHERE id = ?")
            .map_err(|e| DomainError::Internal(e.to_string()))?;
            
        let mut rows = stmt.query(params![id])
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        let current = if let Ok(Some(row)) = rows.next() {
            row.get::<_, bool>(0).unwrap_or(false)
        } else {
            return Err(DomainError::NotFound(format!("Item {} not found", id)));
        };
        // drop rows to free statement borrow?
        // rusqlite rows borrow statement.
        drop(rows);
        drop(stmt);
        
        // Toggle it
        let new_state = !current;
        conn.execute(
            "UPDATE items SET collapsed = ?, updated_at = ? WHERE id = ?",
            params![new_state, chrono::Utc::now().timestamp_millis(), id],
        ).map_err(|e| DomainError::Internal(e.to_string()))?;
        
        Ok(new_state)
    }
}
