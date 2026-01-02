//! Item Repository - Core CRUD Operations
//!
//! SQLite-backed implementation for Item CRUD operations.
//! Specialized operations are in separate modules:
//! - item_hierarchy: Hierarchy operations (children, descendants, move)
//! - item_positioning: Position management
//! - item_workspace: Workspace-specific operations

use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::{Item, ItemType, DomainError, DomainResult};
use super::super::traits::Repository;

/// SQLite implementation of Item repository
pub struct ItemRepository {
    pub(super) conn: Arc<Mutex<Connection>>,
}

impl ItemRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl Repository<Item> for ItemRepository {
    async fn create(&self, entity: &Item) -> DomainResult<Item> {
        // Delegate to create_with_workspace with default workspace ID
        use super::item_workspace::ItemWorkspaceOperations;
        self.create_with_workspace(entity, 1).await
    }

    async fn find_by_id(&self, id: u32) -> DomainResult<Option<Item>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, url, summary, CAST(created_at AS INTEGER) as created_at, CAST(updated_at AS INTEGER) as updated_at FROM items WHERE id = ?",
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
                "SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, url, summary, CAST(created_at AS INTEGER) as created_at, CAST(updated_at AS INTEGER) as updated_at FROM items ORDER BY parent_id NULLS FIRST, position ASC",
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
        // Update item with timestamp
        let conn = self.conn.lock().await;
        
        let text = entity.text.clone();
        let completed = if entity.completed { 1 } else { 0 };
        let item_type = entity.item_type.as_str().to_string();
        let memo = entity.memo.clone();
        let collapsed = if entity.collapsed { 1 } else { 0 };
        let url = entity.url.clone();
        let summary = entity.summary.clone();
        
        conn.execute(
            "UPDATE items SET text = ?, completed = ?, item_type = ?, memo = ?, target_count = ?, current_count = ?, parent_id = ?, position = ?, collapsed = ?, url = ?, summary = ?, updated_at = strftime('%s', 'now') WHERE id = ?",
            libsql::params![
                text,
                completed,
                item_type,
                memo,
                entity.target_count,
                entity.current_count,
                entity.parent_id,
                entity.position,
                collapsed,
                url,
                summary,
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

/// Convert a database row to Item
pub(super) fn row_to_item(row: &libsql::Row) -> DomainResult<Item> {
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
        url: row.get::<Option<String>>(10).ok().flatten(),
        summary: row.get::<Option<String>>(11).ok().flatten(),
        created_at: row.get::<Option<i64>>(12).ok().flatten(),
        updated_at: row.get::<Option<i64>>(13).ok().flatten(),
    })
}
