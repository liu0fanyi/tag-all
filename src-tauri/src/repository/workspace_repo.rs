//! Workspace Repository
//!
//! Handles all workspace-related database operations.

use rusqlite::{Connection, params};
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::domain::{Workspace, DomainResult, DomainError};

/// Fixed workspace IDs (1=todos, 2=files, 3=others, 4=web-bookmarks)
/// These workspaces cannot be deleted or renamed
const FIXED_WORKSPACE_IDS: [u32; 4] = [1, 2, 3, 4];

pub struct WorkspaceRepository {
    conn: Arc<Mutex<Option<Connection>>>,
}

impl WorkspaceRepository {
    pub fn new(conn: Arc<Mutex<Option<Connection>>>) -> Self {
        Self { conn }
    }

    /// List all workspaces
    pub async fn list(&self) -> DomainResult<Vec<Workspace>> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, name FROM workspaces ORDER BY id")
            .map_err(|e| DomainError::Internal(e.to_string()))?;
            
        let mut rows = stmt.query([])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut workspaces = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let id: u32 = row.get(0).unwrap_or(0);
            let name: String = row.get(1).unwrap_or_default();
            workspaces.push(Workspace::new(id, name));
        }
        Ok(workspaces)
    }

    /// Create a new workspace
    pub async fn create(&self, name: &str) -> DomainResult<Workspace> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let now = chrono::Local::now().timestamp_millis();
        conn.execute(
            "INSERT INTO workspaces (name, updated_at) VALUES (?, ?)",
            params![name, now],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let id = conn.last_insert_rowid() as u32;
        Ok(Workspace::new(id, name.to_string()))
    }

    /// Delete a workspace (cannot delete fixed workspaces with IDs 1-4)
    pub async fn delete(&self, id: u32) -> DomainResult<()> {
        if FIXED_WORKSPACE_IDS.contains(&id) {
            return Err(DomainError::InvalidInput("Cannot delete fixed workspace".into()));
        }
        
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        // Delete all items in this workspace first
        conn.execute(
            "DELETE FROM items WHERE workspace_id = ?",
            params![id],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Delete the workspace
        conn.execute(
            "DELETE FROM workspaces WHERE id = ?",
            params![id],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Rename a workspace (cannot rename fixed workspaces with IDs 1-4)
    pub async fn rename(&self, id: u32, name: &str) -> DomainResult<()> {
        if FIXED_WORKSPACE_IDS.contains(&id) {
            return Err(DomainError::InvalidInput("Cannot rename fixed workspace".into()));
        }
        
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let now = chrono::Local::now().timestamp_millis();
        conn.execute(
            "UPDATE workspaces SET name = ?, updated_at = ? WHERE id = ?",
            params![name, now, id],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // Workspace Directory Management
    // ========================================================================

    /// List directory paths for a workspace
    pub async fn list_paths(&self, workspace_id: u32) -> DomainResult<Vec<crate::domain::WorkspaceDir>> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;

        // Updated query to include collapsed
        let mut stmt = conn.prepare("SELECT id, workspace_id, path, collapsed FROM workspace_dirs WHERE workspace_id = ?")
            .map_err(|e| DomainError::Internal(e.to_string()))?;
            
        let mut rows = stmt.query(params![workspace_id])
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut dirs = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let id: u32 = row.get(0).unwrap_or(0);
            let ws_id: u32 = row.get(1).unwrap_or(0);
            let path: String = row.get(2).unwrap_or_default();
            let collapsed: bool = row.get::<_, i32>(3).unwrap_or(1) != 0; // SQLite bool is integer
            
            let mut dir = crate::domain::WorkspaceDir::new(id, ws_id, path);
            dir.collapsed = collapsed;
            dirs.push(dir);
        }
        Ok(dirs)
    }

    /// Add a directory path to a workspace
    pub async fn add_path(&self, workspace_id: u32, path: &str) -> DomainResult<crate::domain::WorkspaceDir> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        // Remove trailing slash for consistency (unless root)
        let clean_path = if path.len() > 3 && (path.ends_with('/') || path.ends_with('\\')) {
            &path[..path.len()-1]
        } else {
            path
        };

        // Check if exists
        let mut stmt = conn.prepare("SELECT id, collapsed FROM workspace_dirs WHERE workspace_id = ? AND path = ?")
             .map_err(|e| DomainError::Internal(e.to_string()))?;
             
        let mut rows = stmt.query(params![workspace_id, clean_path])
            .map_err(|e| DomainError::Internal(e.to_string()))?;
            
        if let Ok(Some(row)) = rows.next() {
             // Already exists, return existing
             let id: u32 = row.get(0).unwrap_or(0);
             let collapsed: bool = row.get::<_, i32>(1).unwrap_or(1) != 0;
             let mut dir = crate::domain::WorkspaceDir::new(id, workspace_id, clean_path.to_string());
             dir.collapsed = collapsed;
             return Ok(dir);
        }

        let now = chrono::Local::now().timestamp_millis();
        conn.execute(
            "INSERT INTO workspace_dirs (workspace_id, path, collapsed, updated_at) VALUES (?, ?, 1, ?)",
            params![workspace_id, clean_path, now],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let id = conn.last_insert_rowid() as u32;
        Ok(crate::domain::WorkspaceDir::new(id, workspace_id, clean_path.to_string()))
    }

    /// Remove a directory path from a workspace
    pub async fn remove_path(&self, id: u32) -> DomainResult<()> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        conn.execute(
            "DELETE FROM workspace_dirs WHERE id = ?",
            params![id],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Update directory collapsed state
    pub async fn set_path_collapsed(&self, id: u32, collapsed: bool) -> DomainResult<()> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or(DomainError::Internal("Database not initialized".to_string()))?;
        
        let now = chrono::Local::now().timestamp_millis();
        let val = if collapsed { 1 } else { 0 };
        
        conn.execute(
            "UPDATE workspace_dirs SET collapsed = ?, updated_at = ? WHERE id = ?",
            params![val, now, id],
        )
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}
