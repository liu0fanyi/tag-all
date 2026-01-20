//! Database Connection and Setup
//!
//! Manages SQLite database connection and migrations.

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Import shared sync types and functions from tauri-sync-db
pub use tauri_sync_db_backend::{
    DbState,
    SyncConfig,
    configure_sync as configure_sync_backend,
    get_sync_config,
    validate_cloud_connection
};

/// Backup data structure for safe migration (tag-all specific)
#[derive(Serialize, Deserialize)]
pub struct BackupData {
    pub items: Vec<serde_json::Value>,
    pub tags: Vec<serde_json::Value>,
    pub workspaces: Vec<serde_json::Value>,
    pub item_tags: Vec<serde_json::Value>,
    pub tag_tags: Vec<serde_json::Value>,
}

// DbState, SyncConfig, validate_cloud_connection are now provided by tauri-sync-db

// Re-export shared crate functions for convenience
pub use tauri_sync_db_backend::configure_sync;

/// Get sync configuration file path (helper for tag-all specific code)
pub fn get_config_path(db_path: &PathBuf) -> PathBuf {
    db_path.parent().unwrap().join("sync_config.json")
}

/// Load sync configuration from file  
pub fn load_config(db_path: &PathBuf) -> Option<SyncConfig> {
    get_sync_config(db_path)
}

/// Initialize database using shared sync crate
pub async fn init_db(db_path: &PathBuf) -> Result<DbState, String> {
    // 1. Initialize DB using shared crate (creates file, sets WAL mode)
    let state = tauri_sync_db_backend::init_db(db_path).await?;
    
    // 2. Run application-specific migrations
    {
        let conn_guard = state.conn.lock().await;
        let conn = conn_guard.as_ref().ok_or("Database connection initialization failed")?;
        
        if let Err(e) = run_migrations(conn) {
            let err_msg = e.to_string();
            eprintln!("Migration failed: {}", err_msg);
            
            // Detailed diagnostics
            let metadata = std::fs::metadata(db_path).map_err(|e| e.to_string())?;
            eprintln!("DB File size (at migration failure): {} bytes", metadata.len());
            
            if metadata.len() > 0 {
                 let integrity: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))
                    .unwrap_or_else(|e| format!("Could not run integrity check: {}", e));
                 
                 return Err(format!("DB Migration failed: {}. Integrity check: {}. File size: {}", err_msg, integrity, metadata.len()));
            } else {
                 return Err(format!("DB Migration failed: {}. File is empty.", err_msg));
            }
        }
    }
    
    Ok(state)
}


/// Check if a column exists in a table
fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let query = format!("PRAGMA table_info({})", table);
    let mut stmt = match conn.prepare(&query) {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    let rows = stmt.query_map([], |row| row.get::<_, String>(1)).ok();
    if let Some(rows) = rows {
        for name_result in rows {
            if let Ok(name) = name_result {
                if name == column {
                    return true;
                }
            }
        }
    }
    false
}

/// Run database migrations
fn run_migrations(conn: &Connection) -> Result<(), String> {
    // Items table - create if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL,
            completed INTEGER NOT NULL DEFAULT 0,
            item_type TEXT NOT NULL DEFAULT 'daily',
            memo TEXT,
            target_count INTEGER,
            current_count INTEGER NOT NULL DEFAULT 0
        )",
        (),
    )
    .map_err(|e| e.to_string())?;

    // Level 2 migrations: Add hierarchy columns if they don't exist
    if !column_exists(conn, "items", "parent_id") {
        conn.execute("ALTER TABLE items ADD COLUMN parent_id INTEGER", ())
            .map_err(|e| format!("Failed to add parent_id: {}", e))?;
    }

    if !column_exists(conn, "items", "position") {
        conn.execute("ALTER TABLE items ADD COLUMN position INTEGER NOT NULL DEFAULT 0", ())
            .map_err(|e| format!("Failed to add position: {}", e))?;
    }

    if !column_exists(conn, "items", "collapsed") {
        conn.execute("ALTER TABLE items ADD COLUMN collapsed INTEGER NOT NULL DEFAULT 0", ())
            .map_err(|e| format!("Failed to add collapsed: {}", e))?;
    }
    
    // Level 6: Add Web Bookmark fields (url, summary, created_at, updated_at)
    if !column_exists(conn, "items", "url") {
        conn.execute("ALTER TABLE items ADD COLUMN url TEXT", ())
            .map_err(|e| format!("Failed to add url: {}", e))?;
    }
    
    if !column_exists(conn, "items", "summary") {
        conn.execute("ALTER TABLE items ADD COLUMN summary TEXT", ())
            .map_err(|e| format!("Failed to add summary: {}", e))?;
    }
    
    if !column_exists(conn, "items", "created_at") {
        conn.execute("ALTER TABLE items ADD COLUMN created_at INTEGER DEFAULT 0", ())
            .map_err(|e| format!("Failed to add created_at: {}", e))?;
    }
    
    if !column_exists(conn, "items", "updated_at") {
        conn.execute("ALTER TABLE items ADD COLUMN updated_at INTEGER DEFAULT 0", ())
            .map_err(|e| format!("Failed to add updated_at: {}", e))?;
    }

    // Level 9: Soft delete support
    if !column_exists(conn, "items", "deleted_at") {
        conn.execute("ALTER TABLE items ADD COLUMN deleted_at INTEGER DEFAULT NULL", ())
            .map_err(|e| format!("Failed to add deleted_at: {}", e))?;
    }

    // Level 7: File Management fields (content_hash, quick_hash, last_known_path, is_dir)
    if !column_exists(conn, "items", "content_hash") {
        conn.execute("ALTER TABLE items ADD COLUMN content_hash TEXT", ())
            .map_err(|e| format!("Failed to add content_hash: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_items_content_hash ON items(content_hash)", ()).map_err(|e| e.to_string())?;
    }

    if !column_exists(conn, "items", "quick_hash") {
        conn.execute("ALTER TABLE items ADD COLUMN quick_hash TEXT", ())
            .map_err(|e| format!("Failed to add quick_hash: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_items_quick_hash ON items(quick_hash)", ()).map_err(|e| e.to_string())?;
    }

    if !column_exists(conn, "items", "last_known_path") {
        conn.execute("ALTER TABLE items ADD COLUMN last_known_path TEXT", ())
            .map_err(|e| format!("Failed to add last_known_path: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_items_path ON items(last_known_path)", ()).map_err(|e| e.to_string())?;
    }

    if !column_exists(conn, "items", "is_dir") {
        conn.execute("ALTER TABLE items ADD COLUMN is_dir INTEGER DEFAULT 0", ())
            .map_err(|e| format!("Failed to add is_dir: {}", e))?;
    }

    // Create index for faster parent-child queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_items_parent ON items(parent_id)",
        (),
    )
    .map_err(|e| e.to_string())?;

    // Level 3: Tags table (with position for root tag ordering)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT,
            position INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER DEFAULT 0
        )",
        (),
    )
    .map_err(|e| e.to_string())?;
    
    // Add position column if missing (migration)
    let _ = conn.execute("ALTER TABLE tags ADD COLUMN position INTEGER DEFAULT 0", ());

    // Add updated_at column if missing (migration)
    if !column_exists(conn, "tags", "updated_at") {
        conn.execute("ALTER TABLE tags ADD COLUMN updated_at INTEGER DEFAULT 0", ())
            .map_err(|e| format!("Failed to add updated_at to tags: {}", e))?;
    }

    // Add created_at and deleted_at for sync support
    if !column_exists(conn, "tags", "created_at") {
        conn.execute("ALTER TABLE tags ADD COLUMN created_at INTEGER DEFAULT 0", ())
            .map_err(|e| format!("Failed to add created_at to tags: {}", e))?;
    }
    if !column_exists(conn, "tags", "deleted_at") {
        conn.execute("ALTER TABLE tags ADD COLUMN deleted_at INTEGER DEFAULT NULL", ())
            .map_err(|e| format!("Failed to add deleted_at to tags: {}", e))?;
    }

    // Level 3: Item-Tag many-to-many relationship
    conn.execute(
        "CREATE TABLE IF NOT EXISTS item_tags (
            item_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            updated_at INTEGER,
            PRIMARY KEY (item_id, tag_id),
            FOREIGN KEY(item_id) REFERENCES items(id) ON DELETE CASCADE,
            FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )",
        (),
    )
    .map_err(|e| e.to_string())?;

    // Add updated_at if missing
    if !column_exists(conn, "item_tags", "updated_at") {
        conn.execute("ALTER TABLE item_tags ADD COLUMN updated_at INTEGER DEFAULT 0", ()).map_err(|e| e.to_string())?;
    }
    // Add created_at and deleted_at for sync support
    if !column_exists(conn, "item_tags", "created_at") {
        conn.execute("ALTER TABLE item_tags ADD COLUMN created_at INTEGER DEFAULT 0", ()).map_err(|e| e.to_string())?;
    }
    if !column_exists(conn, "item_tags", "deleted_at") {
        conn.execute("ALTER TABLE item_tags ADD COLUMN deleted_at INTEGER DEFAULT NULL", ()).map_err(|e| e.to_string())?;
    }

    // Level 3: Tag-Tag multi-parent relationship (tag can have multiple parent tags)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tag_tags (
            child_tag_id INTEGER NOT NULL,
            parent_tag_id INTEGER NOT NULL,
            position INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER,
            PRIMARY KEY (child_tag_id, parent_tag_id),
            FOREIGN KEY(child_tag_id) REFERENCES tags(id) ON DELETE CASCADE,
            FOREIGN KEY(parent_tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )",
        (),
    )
    .map_err(|e| e.to_string())?;

    // Add updated_at if missing
    if !column_exists(conn, "tag_tags", "updated_at") {
        conn.execute("ALTER TABLE tag_tags ADD COLUMN updated_at INTEGER DEFAULT 0", ()).map_err(|e| e.to_string())?;
    }
    // Add created_at and deleted_at for sync support
    if !column_exists(conn, "tag_tags", "created_at") {
        conn.execute("ALTER TABLE tag_tags ADD COLUMN created_at INTEGER DEFAULT 0", ()).map_err(|e| e.to_string())?;
    }
    if !column_exists(conn, "tag_tags", "deleted_at") {
        conn.execute("ALTER TABLE tag_tags ADD COLUMN deleted_at INTEGER DEFAULT NULL", ()).map_err(|e| e.to_string())?;
    }

    // Level 4: Window state persistence
    conn.execute(
        "CREATE TABLE IF NOT EXISTS window_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            width REAL NOT NULL DEFAULT 800,
            height REAL NOT NULL DEFAULT 600,
            x REAL NOT NULL DEFAULT 100,
            y REAL NOT NULL DEFAULT 100,
            pinned INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER DEFAULT 0
        )",
        (),
    )
    .map_err(|e| e.to_string())?;
    
    // Add updated_at to window_state if missing
    if !column_exists(conn, "window_state", "updated_at") {
        conn.execute("ALTER TABLE window_state ADD COLUMN updated_at INTEGER DEFAULT 0", ())
            .map_err(|e| format!("Failed to add updated_at to window_state: {}", e))?;
    }

    // Level 5: Workspaces table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workspaces (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            updated_at INTEGER DEFAULT 0
        )",
        (),
    )
    .map_err(|e| e.to_string())?;

    // Add updated_at to workspaces if missing
    if !column_exists(conn, "workspaces", "updated_at") {
        conn.execute("ALTER TABLE workspaces ADD COLUMN updated_at INTEGER DEFAULT 0", ())
            .map_err(|e| format!("Failed to add updated_at to workspaces: {}", e))?;
    }
    // Add created_at and deleted_at for sync support
    if !column_exists(conn, "workspaces", "created_at") {
        conn.execute("ALTER TABLE workspaces ADD COLUMN created_at INTEGER DEFAULT 0", ())
            .map_err(|e| format!("Failed to add created_at to workspaces: {}", e))?;
    }
    if !column_exists(conn, "workspaces", "deleted_at") {
        conn.execute("ALTER TABLE workspaces ADD COLUMN deleted_at INTEGER DEFAULT NULL", ())
            .map_err(|e| format!("Failed to add deleted_at to workspaces: {}", e))?;
    }

    // Level 7: Workspace Directories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workspace_dirs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            workspace_id INTEGER NOT NULL,
            path TEXT NOT NULL,
            collapsed INTEGER DEFAULT 1,
            updated_at INTEGER DEFAULT 0,
            FOREIGN KEY(workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
        )",
        (),
    )
    .map_err(|e| e.to_string())?;

    // Add collapsed column if missing
    if !column_exists(conn, "workspace_dirs", "collapsed") {
        conn.execute("ALTER TABLE workspace_dirs ADD COLUMN collapsed INTEGER DEFAULT 1", ())
            .map_err(|e| format!("Failed to add collapsed to workspace_dirs: {}", e))?;
    }

    // Add updated_at to workspace_dirs if missing
    if !column_exists(conn, "workspace_dirs", "updated_at") {
        conn.execute("ALTER TABLE workspace_dirs ADD COLUMN updated_at INTEGER DEFAULT 0", ())
            .map_err(|e| format!("Failed to add updated_at to workspace_dirs: {}", e))?;
    }
    // Add created_at and deleted_at for sync support
    if !column_exists(conn, "workspace_dirs", "created_at") {
        conn.execute("ALTER TABLE workspace_dirs ADD COLUMN created_at INTEGER DEFAULT 0", ())
            .map_err(|e| format!("Failed to add created_at to workspace_dirs: {}", e))?;
    }
    if !column_exists(conn, "workspace_dirs", "deleted_at") {
        conn.execute("ALTER TABLE workspace_dirs ADD COLUMN deleted_at INTEGER DEFAULT NULL", ())
            .map_err(|e| format!("Failed to add deleted_at to workspace_dirs: {}", e))?;
    }

    // Add workspace_id column to items if missing
    if !column_exists(conn, "items", "workspace_id") {
        conn.execute("ALTER TABLE items ADD COLUMN workspace_id INTEGER DEFAULT 1", ())
            .map_err(|e| format!("Failed to add workspace_id: {}", e))?;
    }

    // Create 4 fixed workspaces if they don't exist (IDs 1-4 are protected)
    conn.execute(
        "INSERT OR IGNORE INTO workspaces (id, name) VALUES (1, 'todos')",
        (),
    )
    .map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT OR IGNORE INTO workspaces (id, name) VALUES (2, 'files')",
        (),
    )
    .map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT OR IGNORE INTO workspaces (id, name) VALUES (3, 'others')",
        (),
    )
    .map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT OR IGNORE INTO workspaces (id, name) VALUES (4, 'web-bookmarks')",
        (),
    )
    .map_err(|e| e.to_string())?;

    // Migrate existing items without workspace_id to default workspace
    conn.execute(
        "UPDATE items SET workspace_id = 1 WHERE workspace_id IS NULL",
        (),
    )
    .map_err(|e| e.to_string())?;

    // Level 8: Sync Status table (required by generic sync backend)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sync_status (
            table_name TEXT PRIMARY KEY,
            last_sync_time TEXT,
            last_sync_direction TEXT,
            sync_count INTEGER
        )",
        (),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// Data Backup and Restore
// ============================================================================


/// Backup all data from current database
pub fn backup_local_data(conn: &Connection) -> Result<BackupData, String> {
    // Backup items
    // Explicitly select columns to ensure order and completeness
    let mut items_stmt = conn.prepare("SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, workspace_id, url, summary, created_at, updated_at, content_hash, quick_hash, last_known_path, is_dir FROM items").map_err(|e| e.to_string())?;
    let mut items_rows = items_stmt.query([]).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    while let Ok(Some(row)) = items_rows.next() {
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let text: String = row.get(1).map_err(|e| e.to_string())?;
        let completed: i64 = row.get(2).map_err(|e| e.to_string())?;
        let item_type: String = row.get(3).map_err(|e| e.to_string())?;
        let memo: Option<String> = row.get(4).ok();
        let target_count: Option<i64> = row.get(5).ok();
        let current_count: i64 = row.get(6).map_err(|e| e.to_string())?;
        let parent_id: Option<i64> = row.get(7).ok();
        let position: i64 = row.get(8).map_err(|e| e.to_string())?;
        let collapsed: i64 = row.get(9).map_err(|e| e.to_string())?;
        let workspace_id: i64 = row.get(10).map_err(|e| e.to_string())?;
        let url: Option<String> = row.get(11).ok();
        let summary: Option<String> = row.get(12).ok();
        let created_at: Option<i64> = row.get(13).ok();
        let updated_at: Option<i64> = row.get(14).ok();
        let content_hash: Option<String> = row.get(15).ok();
        let quick_hash: Option<String> = row.get(16).ok();
        let last_known_path: Option<String> = row.get(17).ok();
        let is_dir: i64 = row.get::<_, i64>(18).unwrap_or(0);
        
        items.push(serde_json::json!({
            "id": id,
            "text": text,
            "completed": completed,
            "item_type": item_type,
            "memo": memo,
            "target_count": target_count,
            "current_count": current_count,
            "parent_id": parent_id,
            "position": position,
            "collapsed": collapsed,
            "workspace_id": workspace_id,
            "url": url,
            "summary": summary,
            "created_at": created_at,
            "updated_at": updated_at,
            "content_hash": content_hash,
            "quick_hash": quick_hash,
            "last_known_path": last_known_path,
            "is_dir": is_dir
        }));
    }
    
    // Backup tags
    let mut tags_stmt = conn.prepare("SELECT id, name, color, position FROM tags").map_err(|e| e.to_string())?;
    let mut tags_rows = tags_stmt.query([]).map_err(|e| e.to_string())?;
    let mut tags = Vec::new();
    while let Ok(Some(row)) = tags_rows.next() {
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let name: String = row.get(1).map_err(|e| e.to_string())?;
        let color: Option<String> = row.get(2).ok();
        let position: i64 = row.get(3).map_err(|e| e.to_string())?;
        
        tags.push(serde_json::json!({
            "id": id,
            "name": name,
            "color": color,
            "position": position
        }));
    }
    
    // Backup workspaces
    let mut ws_stmt = conn.prepare("SELECT id, name, updated_at FROM workspaces").map_err(|e| e.to_string())?;
    let mut ws_rows = ws_stmt.query([]).map_err(|e| e.to_string())?;
    let mut workspaces = Vec::new();
    while let Ok(Some(row)) = ws_rows.next() {
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let name: String = row.get(1).map_err(|e| e.to_string())?;
        let updated_at: Option<i64> = row.get(2).ok();
        
        workspaces.push(serde_json::json!({
            "id": id,
            "name": name,
            "updated_at": updated_at
        }));
    }

    // Backup item_tags
    let mut item_tags_stmt = conn.prepare("SELECT item_id, tag_id FROM item_tags").map_err(|e| e.to_string())?;
    let mut item_tags_rows = item_tags_stmt.query([]).map_err(|e| e.to_string())?;
    let mut item_tags = Vec::new();
    while let Ok(Some(row)) = item_tags_rows.next() {
        let item_id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let tag_id: i64 = row.get(1).map_err(|e| e.to_string())?;
        
        item_tags.push(serde_json::json!({
            "item_id": item_id,
            "tag_id": tag_id
        }));
    }

    // Backup tag_tags
    let mut tag_tags_stmt = conn.prepare("SELECT child_tag_id, parent_tag_id, position FROM tag_tags").map_err(|e| e.to_string())?;
    let mut tag_tags_rows = tag_tags_stmt.query([]).map_err(|e| e.to_string())?;
    let mut tag_tags = Vec::new();
    while let Ok(Some(row)) = tag_tags_rows.next() {
        let child_tag_id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let parent_tag_id: i64 = row.get(1).map_err(|e| e.to_string())?;
        let position: i64 = row.get(2).map_err(|e| e.to_string())?;
        
        tag_tags.push(serde_json::json!({
            "child_tag_id": child_tag_id,
            "parent_tag_id": parent_tag_id,
            "position": position
        }));
    }
    
    eprintln!("Backup complete: {} items, {} tags, {} workspaces, {} item_tags, {} tag_tags", 
              items.len(), tags.len(), workspaces.len(), item_tags.len(), tag_tags.len());
    
    Ok(BackupData { items, tags, workspaces, item_tags, tag_tags })
}

/// Restore data to database
pub fn restore_data(conn: &Connection, backup: BackupData) -> Result<(), String> {
    eprintln!("Restoring data: {} items, {} tags, {} workspaces, {} item_tags, {} tag_tags",
              backup.items.len(), backup.tags.len(), backup.workspaces.len(), backup.item_tags.len(), backup.tag_tags.len());
    
    // Restore workspaces first
    for ws in backup.workspaces {
        let id = ws["id"].as_i64().unwrap();
        let name = ws["name"].as_str().unwrap();
        let updated_at = ws["updated_at"].as_i64().unwrap_or(0);
        conn.execute(
            "INSERT OR REPLACE INTO workspaces (id, name, updated_at) VALUES (?, ?, ?)",
            params![id, name, updated_at]
        ).map_err(|e| e.to_string())?;
    }
    
    // Restore tags
    for tag in backup.tags {
        let id = tag["id"].as_i64().unwrap();
        let name = tag["name"].as_str().unwrap();
        let color = tag["color"].as_str(); // Option<String>
        let position = tag["position"].as_i64().unwrap_or(0);
        
        conn.execute(
            "INSERT OR REPLACE INTO tags (id, name, color, position) VALUES (?, ?, ?, ?)",
            params![id, name, color, position]
        ).map_err(|e| e.to_string())?;
    }
    
    // Restore items
    for item in backup.items {
        let id = item["id"].as_i64().unwrap();
        let text = item["text"].as_str().unwrap();
        let completed = item["completed"].as_i64().unwrap_or(0);
        let item_type = item["item_type"].as_str().unwrap();
        let memo = item["memo"].as_str();
        let target_count = item["target_count"].as_i64();
        let current_count = item["current_count"].as_i64().unwrap_or(0);
        let parent_id = item["parent_id"].as_i64();
        let position = item["position"].as_i64().unwrap_or(0);
        let collapsed = item["collapsed"].as_i64().unwrap_or(0);
        let workspace_id = item["workspace_id"].as_i64().unwrap_or(1);
        let url = item["url"].as_str();
        let summary = item["summary"].as_str();
        let created_at = item["created_at"].as_i64();
        let updated_at = item["updated_at"].as_i64();
        let content_hash = item["content_hash"].as_str();
        let quick_hash = item["quick_hash"].as_str();
        let last_known_path = item["last_known_path"].as_str();
        let is_dir = item["is_dir"].as_i64().unwrap_or(0);
        
        conn.execute(
            "INSERT OR REPLACE INTO items (id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, workspace_id, url, summary, created_at, updated_at, content_hash, quick_hash, last_known_path, is_dir) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, workspace_id, url, summary, created_at, updated_at, content_hash, quick_hash, last_known_path, is_dir]
        ).map_err(|e| e.to_string())?;
    }

    // Restore item_tags
    for it in backup.item_tags {
        let item_id = it["item_id"].as_i64().unwrap();
        let tag_id = it["tag_id"].as_i64().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO item_tags (item_id, tag_id) VALUES (?, ?)",
            params![item_id, tag_id]
        ).map_err(|e| e.to_string())?;
    }

    // Restore tag_tags
    for tt in backup.tag_tags {
        let child_tag_id = tt["child_tag_id"].as_i64().unwrap();
        let parent_tag_id = tt["parent_tag_id"].as_i64().unwrap();
        let position = tt["position"].as_i64().unwrap_or(0);
        
        conn.execute(
            "INSERT OR REPLACE INTO tag_tags (child_tag_id, parent_tag_id, position) VALUES (?, ?, ?)",
            params![child_tag_id, parent_tag_id, position]
        ).map_err(|e| e.to_string())?;
    }
    
    eprintln!("Data restore complete");
    Ok(())
}
