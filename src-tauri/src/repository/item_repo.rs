//! Item Repository Implementation
//!
//! SQLite-backed implementation of Repository<Item> and HierarchyRepository<Item>

use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::{Item, ItemType, DomainError, DomainResult};
use super::traits::{Repository, HierarchyRepository};

/// SQLite implementation of Item repository
pub struct ItemRepository {
    conn: Arc<Mutex<Connection>>,
}

impl ItemRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Get next position for a parent (used in create)
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
}

#[async_trait]
impl Repository<Item> for ItemRepository {
    async fn create(&self, entity: &Item) -> DomainResult<Item> {
        // Delegate to create_with_workspace with default workspace ID
        self.create_with_workspace(entity, 1).await
    }

    async fn find_by_id(&self, id: u32) -> DomainResult<Option<Item>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed FROM items WHERE id = ?",
                libsql::params![id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Ok(Some(row)) = rows.next().await {
            Ok(Some(row_to_item(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn list(&self) -> DomainResult<Vec<Item>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed FROM items ORDER BY parent_id NULLS FIRST, position ASC",
                (),
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut items = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            items.push(row_to_item(&row)?);
        }
        Ok(items)
    }

    async fn update(&self, entity: &Item) -> DomainResult<Item> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE items SET text = ?, completed = ?, item_type = ?, memo = ?, target_count = ?, current_count = ?, parent_id = ?, position = ?, collapsed = ? WHERE id = ?",
            libsql::params![
                entity.text.clone(),
                if entity.completed { 1 } else { 0 },
                entity.item_type.as_str().to_string(),
                entity.memo.clone(),
                entity.target_count,
                entity.current_count,
                entity.parent_id,
                entity.position,
                if entity.collapsed { 1 } else { 0 },
                entity.id
            ],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(entity.clone())
    }

    async fn delete(&self, id: u32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        // Manual cascade: delete all descendants first
        // Using recursive CTE to get all descendant IDs
        conn.execute(
            "DELETE FROM items WHERE id IN (
                WITH RECURSIVE descendants AS (
                    SELECT id FROM items WHERE parent_id = ?
                    UNION ALL
                    SELECT i.id FROM items i
                    JOIN descendants d ON i.parent_id = d.id
                )
                SELECT id FROM descendants
            )",
            libsql::params![id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        // Delete the item itself
        conn.execute("DELETE FROM items WHERE id = ?", libsql::params![id])
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl HierarchyRepository<Item> for ItemRepository {
    async fn get_children(&self, parent_id: Option<u32>) -> DomainResult<Vec<Item>> {
        let conn = self.conn.lock().await;
        
        let query = match parent_id {
            Some(_) => "SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed FROM items WHERE parent_id = ? ORDER BY position ASC",
            None => "SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed FROM items WHERE parent_id IS NULL ORDER BY position ASC",
        };
        
        let mut rows = match parent_id {
            Some(pid) => conn.query(query, libsql::params![pid]).await,
            None => conn.query(query, ()).await,
        }.map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut items = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            items.push(row_to_item(&row)?);
        }
        Ok(items)
    }

    async fn move_to(&self, id: u32, new_parent_id: Option<u32>, position: i32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        // Make room by shifting existing items at target position down
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

        Ok(())
    }

    async fn get_descendants(&self, id: u32) -> DomainResult<Vec<Item>> {
        let conn = self.conn.lock().await;
        
        // Recursive CTE to get all descendants
        let mut rows = conn
            .query(
                "WITH RECURSIVE descendants AS (
                    SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed
                    FROM items WHERE parent_id = ?
                    UNION ALL
                    SELECT i.id, i.text, i.completed, i.item_type, i.memo, i.target_count, i.current_count, i.parent_id, i.position, i.collapsed
                    FROM items i
                    JOIN descendants d ON i.parent_id = d.id
                )
                SELECT * FROM descendants ORDER BY parent_id, position",
                libsql::params![id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut items = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            items.push(row_to_item(&row)?);
        }
        Ok(items)
    }

    async fn toggle_collapsed(&self, id: u32) -> DomainResult<bool> {
        let conn = self.conn.lock().await;
        
        // Toggle and return new value
        conn.execute(
            "UPDATE items SET collapsed = NOT collapsed WHERE id = ?",
            libsql::params![id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Get the new value
        let mut rows = conn
            .query("SELECT collapsed FROM items WHERE id = ?", libsql::params![id])
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Ok(Some(row)) = rows.next().await {
            Ok(row.get::<i32>(0).unwrap_or(0) != 0)
        } else {
            Err(DomainError::NotFound(format!("Item {} not found", id)))
        }
    }
}

/// Convert a database row to Item
fn row_to_item(row: &libsql::Row) -> DomainResult<Item> {
    Ok(Item {
        id: row.get::<u32>(0).map_err(|e| DomainError::Internal(e.to_string()))?,
        text: row.get::<String>(1).map_err(|e| DomainError::Internal(e.to_string()))?,
        completed: row.get::<i32>(2).map_err(|e| DomainError::Internal(e.to_string()))? != 0,
        item_type: ItemType::from_str(&row.get::<String>(3).unwrap_or_else(|_| "daily".to_string())),
        memo: row.get::<Option<String>>(4).ok().flatten(),
        target_count: row.get::<Option<i32>>(5).ok().flatten(),
        current_count: row.get::<i32>(6).unwrap_or(0),
        parent_id: row.get::<Option<u32>>(7).ok().flatten(),
        position: row.get::<i32>(8).unwrap_or(0),
        collapsed: row.get::<i32>(9).unwrap_or(0) != 0,
    })
}

/// Additional ItemRepository methods
impl ItemRepository {
    /// List items by workspace
    pub async fn list_by_workspace(&self, workspace_id: u32) -> DomainResult<Vec<Item>> {
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
            items.push(row_to_item(&row)?);
        }
        Ok(items)
    }

    /// Create item with specific workspace_id
    pub async fn create_with_workspace(&self, entity: &Item, workspace_id: u32) -> DomainResult<Item> {
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

    /// Reset all completed items in a workspace back to incomplete
    /// Returns the count of items reset
    pub async fn reset_all_completed(&self, workspace_id: u32) -> DomainResult<u32> {
        let conn = self.conn.lock().await;
        
        // Reset completed flag to false for all completed items in the workspace
        // Also reset current_count to target_count for countdown items
        conn.execute(
            "UPDATE items SET completed = 0, current_count = COALESCE(target_count, current_count) WHERE workspace_id = ? AND completed = 1",
            libsql::params![workspace_id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Return the number of affected rows (approximate via a count query)
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
