//! Item Workspace Operations
//!
//! Operations for managing items within specific workspaces.

use async_trait::async_trait;

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
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed FROM items WHERE workspace_id = ? ORDER BY parent_id NULLS FIRST, position ASC",
                libsql::params![workspace_id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut items = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            items.push(super::item_repo::row_to_item(&row)?);
        }
        Ok(items)
    }

    async fn create_with_workspace(&self, entity: &Item, workspace_id: u32) -> DomainResult<Item> {
        let conn = self.conn.lock().await;
        
        // Calculate position in same connection
        let position = if entity.position == 0 {
            let query = match entity.parent_id {
                Some(pid) => format!(
                    "SELECT COALESCE(MAX(position), -1) + 1 FROM items WHERE parent_id = {} AND workspace_id = {}", pid, workspace_id
                ),
                None => format!("SELECT COALESCE(MAX(position), -1) + 1 FROM items WHERE parent_id IS NULL AND workspace_id = {}", workspace_id),
            };
            
            let mut rows = conn.query(&query, ())
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            
            if let Ok(Some(row)) = rows.next().await {
                row.get::<i32>(0).unwrap_or(0)
            } else {
                0
            }
        } else {
            entity.position
        };
        
        conn.execute(
            "INSERT INTO items (text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, workspace_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                entity.text.clone(),
                if entity.completed { 1 } else { 0 },
                entity.item_type.as_str().to_string(),
                entity.memo.clone(),
                entity.target_count,
                entity.current_count,
                entity.parent_id,
                position,
                if entity.collapsed { 1 } else { 0 },
                workspace_id
            ],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let id = conn.last_insert_rowid() as u32;
        
        let mut item = entity.clone();
        item.id = id;
        item.position = position;
        Ok(item)
    }

    async fn reset_all_completed(&self, workspace_id: u32) -> DomainResult<u32> {
        let conn = self.conn.lock().await;
        
        // Reset completed flag to false for all completed items in the workspace
        conn.execute(
            "UPDATE items SET completed = 0 WHERE workspace_id = ? AND completed = 1",
            libsql::params![workspace_id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Return the number of affected rows
        let mut rows = conn
            .query(
                "SELECT changes()",
                (),
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        if let Ok(Some(row)) = rows.next().await {
            Ok(row.get::<u32>(0).unwrap_or(0))
        } else {
            Ok(0)
        }
    }
}
