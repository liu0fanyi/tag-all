//! Item Repository - Core CRUD Operations
//!
//! SQLite-backed implementation for Item CRUD operations.
//! Specialized operations are in separate modules:
//! - item_hierarchy: Hierarchy operations (children, descendants, move)
//! - item_positioning: Position management
//! - item_workspace: Workspace-specific operations

use async_trait::async_trait;
use rusqlite::{Connection, params};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::str::FromStr;

use crate::domain::{Item, ItemType, DomainError, DomainResult};
use super::super::traits::Repository;

/// SQLite implementation of Item repository
pub struct ItemRepository {
    pub(super) conn: Arc<Mutex<Option<Connection>>>,
}

impl ItemRepository {
    pub fn new(conn: Arc<Mutex<Option<Connection>>>) -> Self {
        Self { conn }
    }

    pub async fn find_by_last_known_path(&self, path: &str) -> DomainResult<Option<Item>> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, url, summary, CAST(created_at AS INTEGER) as created_at, CAST(updated_at AS INTEGER) as updated_at, content_hash, quick_hash, last_known_path, is_dir FROM items WHERE last_known_path = ?")
            .map_err(|e| DomainError::Internal(e.to_string()))?;
            
        let mut rows = stmt.query(params![path])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Ok(Some(row)) = rows.next() {
            Ok(Some(row_to_item(&row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn find_by_quick_hash(&self, quick_hash: &str, is_dir: bool) -> DomainResult<Option<Item>> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, url, summary, CAST(created_at AS INTEGER) as created_at, CAST(updated_at AS INTEGER) as updated_at, content_hash, quick_hash, last_known_path, is_dir FROM items WHERE quick_hash = ? AND is_dir = ?")
             .map_err(|e| DomainError::Internal(e.to_string()))?;
             
        let mut rows = stmt.query(params![quick_hash, if is_dir { 1 } else { 0 }])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Ok(Some(row)) = rows.next() {
            Ok(Some(row_to_item(&row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn find_by_content_hash(&self, content_hash: &str) -> DomainResult<Option<Item>> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, url, summary, CAST(created_at AS INTEGER) as created_at, CAST(updated_at AS INTEGER) as updated_at, content_hash, quick_hash, last_known_path, is_dir FROM items WHERE content_hash = ?")
             .map_err(|e| DomainError::Internal(e.to_string()))?;
             
        let mut rows = stmt.query(params![content_hash])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Ok(Some(row)) = rows.next() {
            Ok(Some(row_to_item(&row)?))
        } else {
            Ok(None)
        }
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
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, url, summary, CAST(created_at AS INTEGER) as created_at, CAST(updated_at AS INTEGER) as updated_at, content_hash, quick_hash, last_known_path, is_dir FROM items WHERE id = ?")
            .map_err(|e| DomainError::Internal(e.to_string()))?;
            
        let mut rows = stmt.query(params![id])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Ok(Some(row)) = rows.next() {
            Ok(Some(row_to_item(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn list(&self) -> DomainResult<Vec<Item>> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, url, summary, CAST(created_at AS INTEGER) as created_at, CAST(updated_at AS INTEGER) as updated_at, content_hash, quick_hash, last_known_path, is_dir FROM items ORDER BY parent_id NULLS FIRST, position ASC")
             .map_err(|e| DomainError::Internal(e.to_string()))?;
             
        let mut rows = stmt.query([])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut items = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            items.push(row_to_item(&row)?);
        }
        Ok(items)
    }

    async fn update(&self, entity: &Item) -> DomainResult<Item> {
        // Update item with timestamp
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let text = entity.text.clone();
        let completed = if entity.completed { 1 } else { 0 };
        let item_type = entity.item_type.as_str().to_string();
        let memo = entity.memo.clone();
        let collapsed = if entity.collapsed { 1 } else { 0 };
        let url = entity.url.clone();
        let summary = entity.summary.clone();
        let is_dir = if entity.is_dir { 1 } else { 0 };
        let now = chrono::Utc::now().timestamp_millis();
        
        conn.execute(
            "UPDATE items SET text = ?, completed = ?, item_type = ?, memo = ?, target_count = ?, current_count = ?, parent_id = ?, position = ?, collapsed = ?, url = ?, summary = ?, content_hash = ?, quick_hash = ?, last_known_path = ?, is_dir = ?, updated_at = ? WHERE id = ?",
            params![
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
                entity.content_hash.clone(),
                entity.quick_hash.clone(),
                entity.last_known_path.clone(),
                is_dir,
                now,
                entity.id
            ],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut updated_item = entity.clone();
        updated_item.updated_at = Some(now);
        Ok(updated_item)
    }

    async fn delete(&self, id: u32) -> DomainResult<()> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
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
            params![id],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        // Delete the item itself
        conn.execute("DELETE FROM items WHERE id = ?", params![id])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

/// Convert a database row to Item
pub(super) fn row_to_item(row: &rusqlite::Row) -> DomainResult<Item> {
    Ok(Item {
        id: row.get::<_, u32>(0).map_err(|e: rusqlite::Error| DomainError::Internal(e.to_string()))?,
        text: row.get::<_, String>(1).map_err(|e: rusqlite::Error| DomainError::Internal(e.to_string()))?,
        completed: row.get::<_, i32>(2).map_err(|e: rusqlite::Error| DomainError::Internal(e.to_string()))? != 0,
        item_type: ItemType::from_str(&row.get::<_, String>(3).unwrap_or_else(|_| "daily".to_string())),
        memo: row.get::<_, Option<String>>(4).unwrap_or(None),
        target_count: row.get::<_, Option<i32>>(5).unwrap_or(None),
        current_count: row.get::<_, i32>(6).unwrap_or(0),
        parent_id: row.get::<_, Option<u32>>(7).unwrap_or(None),
        position: row.get::<_, i32>(8).unwrap_or(0),
        collapsed: row.get::<_, i32>(9).unwrap_or(0) != 0,
        url: row.get::<_, Option<String>>(10).unwrap_or(None),
        summary: row.get::<_, Option<String>>(11).unwrap_or(None),
        created_at: row.get::<_, Option<i64>>(12).unwrap_or(None),
        updated_at: row.get::<_, Option<i64>>(13).unwrap_or(None),
        content_hash: row.get::<_, Option<String>>(14).unwrap_or(None),
        quick_hash: row.get::<_, Option<String>>(15).unwrap_or(None),
        last_known_path: row.get::<_, Option<String>>(16).unwrap_or(None),
        is_dir: row.get::<_, i32>(17).unwrap_or(0) != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use crate::domain::{Item, ItemType};
    use crate::repository::traits::Repository;

    async fn setup_repo() -> ItemRepository {
        let conn = Connection::open_in_memory().unwrap();
        // Create table matching schema
        conn.execute(
            "CREATE TABLE items (
                id INTEGER PRIMARY KEY,
                text TEXT NOT NULL,
                completed INTEGER DEFAULT 0,
                item_type TEXT DEFAULT 'daily',
                memo TEXT,
                target_count INTEGER,
                current_count INTEGER DEFAULT 0,
                parent_id INTEGER,
                workspace_id INTEGER DEFAULT 1,
                position INTEGER DEFAULT 0,
                collapsed INTEGER DEFAULT 0,
                url TEXT,
                summary TEXT,
                created_at INTEGER,
                updated_at INTEGER,
                content_hash TEXT,
                quick_hash TEXT,
                last_known_path TEXT,
                is_dir INTEGER DEFAULT 0
            )",
            [],
        )
        .unwrap();
        
        ItemRepository::new(Arc::new(Mutex::new(Some(conn))))
    }

    #[tokio::test]
    async fn test_create_and_update_timestamp() {
        let repo = setup_repo().await;
        
        // 1. Create Item
        let item = Item::new(0, "Test Item".to_string(), ItemType::Daily);
        
        // We use repository create, which delegates to create_with_workspace(1)
        let created = repo.create(&item).await.unwrap();
        
        let now = chrono::Utc::now().timestamp_millis();
        
        // created_at should be set
        assert!(created.created_at.is_some());
        let created_at = created.created_at.unwrap();
        
        // Check created_at is valid (within last 5 seconds)
        assert!(created_at > now - 5000 && created_at <= now + 100);
        
        // updated_at should match created_at initially
        assert!(created.updated_at.is_some());
        assert_eq!(created.updated_at.unwrap(), created_at);
        
        // Sleep slightly to guarantee different timestamp (needs async sleep or std sleep if single threaded)
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // 2. Update Item
        let mut to_update = created.clone();
        to_update.text = "Updated Name".to_string();
        
        // Update uses the generic update, which sets updated_at
        let updated = repo.update(&to_update).await.unwrap();
        
        let updated_ts = updated.updated_at.unwrap();
        assert!(updated_ts > created_at, "Updated time {} should be > created time {}", updated_ts, created_at);
        assert_eq!(updated.text, "Updated Name");
    }
}
