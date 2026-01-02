//! Workspace Repository
//!
//! Handles all workspace-related database operations.

use libsql::Connection;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::domain::{Workspace, DomainResult, DomainError};

/// Fixed workspace IDs (1=todos, 2=files, 3=others, 4=web-bookmarks)
/// These workspaces cannot be deleted or renamed
const FIXED_WORKSPACE_IDS: [u32; 4] = [1, 2, 3, 4];

pub struct WorkspaceRepository {
    conn: Arc<Mutex<Connection>>,
}

impl WorkspaceRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// List all workspaces
    pub async fn list(&self) -> DomainResult<Vec<Workspace>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query("SELECT id, name FROM workspaces ORDER BY id", ())
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut workspaces = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            let id: u32 = row.get::<u32>(0).unwrap_or(0);
            let name: String = row.get::<String>(1).unwrap_or_default();
            workspaces.push(Workspace::new(id, name));
        }
        Ok(workspaces)
    }

    /// Create a new workspace
    pub async fn create(&self, name: &str) -> DomainResult<Workspace> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT INTO workspaces (name) VALUES (?)",
            libsql::params![name],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let id = conn.last_insert_rowid() as u32;
        Ok(Workspace::new(id, name.to_string()))
    }

    /// Delete a workspace (cannot delete fixed workspaces with IDs 1-4)
    pub async fn delete(&self, id: u32) -> DomainResult<()> {
        if FIXED_WORKSPACE_IDS.contains(&id) {
            return Err(DomainError::InvalidInput("Cannot delete fixed workspace".into()));
        }
        
        let conn = self.conn.lock().await;
        
        // Delete all items in this workspace first
        conn.execute(
            "DELETE FROM items WHERE workspace_id = ?",
            libsql::params![id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Delete the workspace
        conn.execute(
            "DELETE FROM workspaces WHERE id = ?",
            libsql::params![id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Rename a workspace (cannot rename fixed workspaces with IDs 1-4)
    pub async fn rename(&self, id: u32, name: &str) -> DomainResult<()> {
        if FIXED_WORKSPACE_IDS.contains(&id) {
            return Err(DomainError::InvalidInput("Cannot rename fixed workspace".into()));
        }
        
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE workspaces SET name = ? WHERE id = ?",
            libsql::params![name, id],
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}
