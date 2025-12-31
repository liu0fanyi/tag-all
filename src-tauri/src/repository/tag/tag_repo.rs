//! Tag Repository - Core CRUD Operations
//!
//! SQLite-backed implementation for Tag CRUD operations.
//! Specialized operations are in separate modules:
//! - item_tag: Item-Tag relationships
//! - tag_hierarchy: Tag-Tag relationships (parent-child)
//! - tag_positioning: Position management

use async_trait::async_trait;
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::{Tag, DomainError, DomainResult};
use super::super::traits::Repository;

/// SQLite implementation of Tag repository
pub struct TagRepository {
    pub(super) conn: Arc<Mutex<Connection>>,
}

impl TagRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl Repository<Tag> for TagRepository {
    async fn create(&self, entity: &Tag) -> DomainResult<Tag> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT INTO tags (name, color) VALUES (?, ?)",
            libsql::params![entity.name.clone(), entity.color.clone()],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let id = conn.last_insert_rowid() as u32;
        
        let mut tag = entity.clone();
        tag.id = id;
        Ok(tag)
    }

    async fn find_by_id(&self, id: u32) -> DomainResult<Option<Tag>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query(
                "SELECT id, name, color FROM tags WHERE id = ?",
                libsql::params![id],
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Ok(Some(row)) = rows.next().await {
            Ok(Some(row_to_tag(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn list(&self) -> DomainResult<Vec<Tag>> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query("SELECT id, name, color FROM tags ORDER BY name", ())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut tags = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            tags.push(row_to_tag(&row)?);
        }
        Ok(tags)
    }

    async fn update(&self, entity: &Tag) -> DomainResult<Tag> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE tags SET name = ?, color = ? WHERE id = ?",
            libsql::params![entity.name.clone(), entity.color.clone(), entity.id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(entity.clone())
    }

    async fn delete(&self, id: u32) -> DomainResult<()> {
        let conn = self.conn.lock().await;
        
        // CASCADE will remove item_tags entries
        conn.execute("DELETE FROM tags WHERE id = ?", libsql::params![id])
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

/// Convert a database row to Tag
pub(super) fn row_to_tag(row: &libsql::Row) -> DomainResult<Tag> {
    Ok(Tag {
        id: row.get::<u32>(0).map_err(|e| DomainError::Internal(e.to_string()))?,
        name: row.get::<String>(1).map_err(|e| DomainError::Internal(e.to_string()))?,
        color: row.get::<Option<String>>(2).ok().flatten(),
        position: row.get::<i32>(3).unwrap_or(0),
    })
}
