//! Tag Repository - Core CRUD Operations
//!
//! SQLite-backed implementation for Tag CRUD operations.
//! Specialized operations are in separate modules:
//! - item_tag: Item-Tag relationships
//! - tag_hierarchy: Tag-Tag relationships (parent-child)
//! - tag_positioning: Position management

use async_trait::async_trait;
use rusqlite::{Connection, params};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::{Tag, DomainError, DomainResult};
use super::super::traits::Repository;

/// SQLite implementation of Tag repository
pub struct TagRepository {
    pub(super) conn: Arc<Mutex<Option<Connection>>>,
}

impl TagRepository {
    pub fn new(conn: Arc<Mutex<Option<Connection>>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl Repository<Tag> for TagRepository {
    async fn create(&self, entity: &Tag) -> DomainResult<Tag> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        conn.execute(
            "INSERT INTO tags (name, color, updated_at) VALUES (?, ?, ?)",
            params![entity.name.clone(), entity.color.clone(), chrono::Utc::now().timestamp_millis()],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let id = conn.last_insert_rowid() as u32;
        
        let mut tag = entity.clone();
        tag.id = id;
        Ok(tag)
    }

    async fn find_by_id(&self, id: u32) -> DomainResult<Option<Tag>> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, name, color, position FROM tags WHERE id = ? AND deleted_at IS NULL")
            .map_err(|e| DomainError::Internal(e.to_string()))?;
            
        let mut rows = stmt.query(params![id])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Ok(Some(row)) = rows.next() {
            Ok(Some(row_to_tag(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn list(&self) -> DomainResult<Vec<Tag>> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, name, color, position FROM tags WHERE deleted_at IS NULL ORDER BY name")
             .map_err(|e| DomainError::Internal(e.to_string()))?;
             
        let mut rows = stmt.query([])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut tags = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            tags.push(row_to_tag(&row)?);
        }
        Ok(tags)
    }

    async fn update(&self, entity: &Tag) -> DomainResult<Tag> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        conn.execute(
            "UPDATE tags SET name = ?, color = ?, updated_at = ? WHERE id = ?",
            params![entity.name.clone(), entity.color.clone(), chrono::Utc::now().timestamp_millis(), entity.id],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(entity.clone())
    }

    async fn delete(&self, id: u32) -> DomainResult<()> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let now = chrono::Utc::now().timestamp_millis();
        
        // Soft delete: set deleted_at instead of removing
        conn.execute(
            "UPDATE tags SET deleted_at = ?, updated_at = ? WHERE id = ?",
            params![now, now, id],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

/// Convert a database row to Tag
pub(super) fn row_to_tag(row: &rusqlite::Row) -> DomainResult<Tag> {
    Ok(Tag {
        id: row.get(0).map_err(|e: rusqlite::Error| DomainError::Internal(e.to_string()))?,
        name: row.get(1).map_err(|e: rusqlite::Error| DomainError::Internal(e.to_string()))?,
        color: row.get::<_, Option<String>>(2).unwrap_or(None),
        position: row.get::<_, i32>(3).unwrap_or(0),
    })
}
