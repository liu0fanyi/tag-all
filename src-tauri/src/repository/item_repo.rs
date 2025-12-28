//! Item Repository Implementation
//!
//! SQLite-backed implementation of Repository<Item>

use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::{Item, ItemType, DomainError, DomainResult};
use super::traits::Repository;

/// SQLite implementation of Item repository
pub struct ItemRepository {
    conn: Arc<Mutex<Connection>>,
}

impl ItemRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl Repository<Item> for ItemRepository {
    async fn create(&self, entity: &Item) -> DomainResult<Item> {
        let conn = self.conn.lock().await;
        
        // Use libsql params! macro style - pass values directly
        conn.execute(
            "INSERT INTO items (text, completed, item_type, memo, target_count, current_count) VALUES (?, ?, ?, ?, ?, ?)",
            libsql::params![
                entity.text.clone(),
                if entity.completed { 1 } else { 0 },
                entity.item_type.as_str().to_string(),
                entity.memo.clone(),
                entity.target_count,
                entity.current_count
            ],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let id = conn.last_insert_rowid() as u32;
        
        let mut item = entity.clone();
        item.id = id;
        Ok(item)
    }

    async fn find_by_id(&self, id: u32) -> DomainResult<Option<Item>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT id, text, completed, item_type, memo, target_count, current_count FROM items WHERE id = ?",
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
                "SELECT id, text, completed, item_type, memo, target_count, current_count FROM items ORDER BY id ASC",
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
            "UPDATE items SET text = ?, completed = ?, item_type = ?, memo = ?, target_count = ?, current_count = ? WHERE id = ?",
            libsql::params![
                entity.text.clone(),
                if entity.completed { 1 } else { 0 },
                entity.item_type.as_str().to_string(),
                entity.memo.clone(),
                entity.target_count,
                entity.current_count,
                entity.id
            ],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(entity.clone())
    }

    async fn delete(&self, id: u32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        conn.execute("DELETE FROM items WHERE id = ?", libsql::params![id])
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
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
    })
}
