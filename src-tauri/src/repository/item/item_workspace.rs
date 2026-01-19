//! Item Workspace Operations
//!
//! Operations for managing items within specific workspaces.

use async_trait::async_trait;
use rusqlite::params;

use crate::domain::{Item, DomainError, DomainResult};

/// Trait for workspace-specific item operations
#[async_trait]
pub trait ItemWorkspaceOperations {
    /// List items by workspace
    async fn list_by_workspace(&self, workspace_id: u32) -> DomainResult<Vec<Item>>;
    
    /// Create item with specific workspace_id
    async fn create_with_workspace(&self, entity: &Item, workspace_id: u32) -> DomainResult<Item>;
    
    /// Reset all completed items in a workspace back to incomplete
    async fn reset_all_completed(&self, workspace_id: u32) -> DomainResult<u32>;
}

#[async_trait]
impl ItemWorkspaceOperations for super::item_repo::ItemRepository {
    async fn list_by_workspace(&self, workspace_id: u32) -> DomainResult<Vec<Item>> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, url, summary, CAST(created_at AS INTEGER) as created_at, CAST(updated_at AS INTEGER) as updated_at, content_hash, quick_hash, last_known_path, is_dir FROM items WHERE workspace_id = ? ORDER BY parent_id NULLS FIRST, position ASC")
             .map_err(|e| DomainError::Internal(e.to_string()))?;
             
        let mut rows = stmt.query(params![workspace_id])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut items = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            items.push(super::item_repo::row_to_item(&row)?);
        }
        Ok(items)
    }

    async fn create_with_workspace(&self, entity: &Item, workspace_id: u32) -> DomainResult<Item> {
        let guard = self.conn.lock().await;
        // Block for initial query to calculate position
        let position = {
            let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
            
            if entity.position == 0 {
                let query = match entity.parent_id {
                    Some(pid) => format!(
                        "SELECT COALESCE(MAX(position), -1) + 1 FROM items WHERE parent_id = {} AND workspace_id = {}", pid, workspace_id
                    ),
                    None => format!("SELECT COALESCE(MAX(position), -1) + 1 FROM items WHERE parent_id IS NULL AND workspace_id = {}", workspace_id),
                };
                
                let mut stmt = conn.prepare(&query)
                    .map_err(|e| DomainError::Internal(e.to_string()))?;
                let mut rows = stmt.query([])
                    .map_err(|e| DomainError::Internal(e.to_string()))?;
                
                if let Ok(Some(row)) = rows.next() {
                    row.get::<_, i32>(0).unwrap_or(0)
                } else {
                    0
                }
            } else {
                entity.position
            }
        };

        {
             let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
             
             let is_dir = if entity.is_dir { 1 } else { 0 };
             let now = chrono::Utc::now().timestamp_millis();
        
             conn.execute(
                "INSERT INTO items (text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, workspace_id, url, summary, content_hash, quick_hash, last_known_path, is_dir, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    entity.text.clone(),
                    if entity.completed { 1 } else { 0 },
                    entity.item_type.as_str().to_string(),
                    entity.memo.clone(),
                    entity.target_count,
                    entity.current_count,
                    entity.parent_id,
                    position,
                    if entity.collapsed { 1 } else { 0 },
                    workspace_id,
                    entity.url.clone(),
                    entity.summary.clone(),
                    entity.content_hash.clone(),
                    entity.quick_hash.clone(),
                    entity.last_known_path.clone(),
                    is_dir,
                    now,
                    now
                ],
            )
            .map_err(|e| DomainError::Internal(e.to_string()))?;
            
            // Return item with ID and timestamps
            let id = conn.last_insert_rowid() as u32;
            let mut item = entity.clone();
            item.id = id;
            item.position = position;
            item.created_at = Some(now);
            item.updated_at = Some(now);
            Ok(item)
        }
    }

    async fn reset_all_completed(&self, workspace_id: u32) -> DomainResult<u32> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        // Reset completed flag to false for all completed items in the workspace
        conn.execute(
            "UPDATE items SET completed = 0, updated_at = ? WHERE workspace_id = ? AND completed = 1",
            params![chrono::Utc::now().timestamp_millis(), workspace_id],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Return the number of affected rows
        // SQLite: changes() returns number of rows modified by last INSERT/UPDATE/DELETE.
        let mut stmt = conn.prepare("SELECT changes()")
             .map_err(|e| DomainError::Internal(e.to_string()))?;
             
        let mut rows = stmt.query([])
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        if let Ok(Some(row)) = rows.next() {
            Ok(row.get::<_, u32>(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }
}
