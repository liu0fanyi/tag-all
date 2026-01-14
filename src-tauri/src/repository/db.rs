//! Database Connection and Setup
//!
//! Manages SQLite database connection and migrations.

use libsql::{Builder, Connection};
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
    // Use shared crate's init_db with tag-all specific migrations
    tauri_sync_db_backend::init_db(db_path, |conn| {
        Box::pin(async {
            run_migrations(conn).await
        })
    }).await
}


/// Check if a column exists in a table
async fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let query = format!("PRAGMA table_info({})", table);
    if let Ok(mut rows) = conn.query(&query, ()).await {
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(name) = row.get::<String>(1) {
                if name == column {
                    return true;
                }
            }
        }
    }
    false
}

/// Run database migrations
async fn run_migrations(conn: &Connection) -> Result<(), String> {
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
    .await
    .map_err(|e| e.to_string())?;

    // Level 2 migrations: Add hierarchy columns if they don't exist
    if !column_exists(conn, "items", "parent_id").await {
        conn.execute("ALTER TABLE items ADD COLUMN parent_id INTEGER", ())
            .await
            .map_err(|e| format!("Failed to add parent_id: {}", e))?;
    }

    if !column_exists(conn, "items", "position").await {
        conn.execute("ALTER TABLE items ADD COLUMN position INTEGER NOT NULL DEFAULT 0", ())
            .await
            .map_err(|e| format!("Failed to add position: {}", e))?;
    }

    if !column_exists(conn, "items", "collapsed").await {
        conn.execute("ALTER TABLE items ADD COLUMN collapsed INTEGER NOT NULL DEFAULT 0", ())
            .await
            .map_err(|e| format!("Failed to add collapsed: {}", e))?;
    }
    
    // Level 6: Add Web Bookmark fields (url, summary, created_at, updated_at)
    if !column_exists(conn, "items", "url").await {
        conn.execute("ALTER TABLE items ADD COLUMN url TEXT", ())
            .await
            .map_err(|e| format!("Failed to add url: {}", e))?;
    }
    
    if !column_exists(conn, "items", "summary").await {
        conn.execute("ALTER TABLE items ADD COLUMN summary TEXT", ())
            .await
            .map_err(|e| format!("Failed to add summary: {}", e))?;
    }
    
    if !column_exists(conn, "items", "created_at").await {
        conn.execute("ALTER TABLE items ADD COLUMN created_at INTEGER DEFAULT 0", ())
            .await
            .map_err(|e| format!("Failed to add created_at: {}", e))?;
    }
    
    if !column_exists(conn, "items", "updated_at").await {
        conn.execute("ALTER TABLE items ADD COLUMN updated_at INTEGER DEFAULT 0", ())
            .await
            .map_err(|e| format!("Failed to add updated_at: {}", e))?;
    }

    // Level 7: File Management fields (content_hash, quick_hash, last_known_path, is_dir)
    if !column_exists(conn, "items", "content_hash").await {
        conn.execute("ALTER TABLE items ADD COLUMN content_hash TEXT", ())
            .await
            .map_err(|e| format!("Failed to add content_hash: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_items_content_hash ON items(content_hash)", ()).await.map_err(|e| e.to_string())?;
    }

    if !column_exists(conn, "items", "quick_hash").await {
        conn.execute("ALTER TABLE items ADD COLUMN quick_hash TEXT", ())
            .await
            .map_err(|e| format!("Failed to add quick_hash: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_items_quick_hash ON items(quick_hash)", ()).await.map_err(|e| e.to_string())?;
    }

    if !column_exists(conn, "items", "last_known_path").await {
        conn.execute("ALTER TABLE items ADD COLUMN last_known_path TEXT", ())
            .await
            .map_err(|e| format!("Failed to add last_known_path: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_items_path ON items(last_known_path)", ()).await.map_err(|e| e.to_string())?;
    }

    if !column_exists(conn, "items", "is_dir").await {
        conn.execute("ALTER TABLE items ADD COLUMN is_dir INTEGER DEFAULT 0", ())
            .await
            .map_err(|e| format!("Failed to add is_dir: {}", e))?;
    }

    // Create index for faster parent-child queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_items_parent ON items(parent_id)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Level 3: Tags table (with position for root tag ordering)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT,
            position INTEGER NOT NULL DEFAULT 0
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;
    
    // Add position column if missing (migration)
    let _ = conn.execute("ALTER TABLE tags ADD COLUMN position INTEGER DEFAULT 0", ()).await;

    // Level 3: Item-Tag many-to-many relationship
    conn.execute(
        "CREATE TABLE IF NOT EXISTS item_tags (
            item_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (item_id, tag_id),
            FOREIGN KEY(item_id) REFERENCES items(id) ON DELETE CASCADE,
            FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Level 3: Tag-Tag multi-parent relationship (tag can have multiple parent tags)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tag_tags (
            child_tag_id INTEGER NOT NULL,
            parent_tag_id INTEGER NOT NULL,
            position INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (child_tag_id, parent_tag_id),
            FOREIGN KEY(child_tag_id) REFERENCES tags(id) ON DELETE CASCADE,
            FOREIGN KEY(parent_tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Level 4: Window state persistence
    conn.execute(
        "CREATE TABLE IF NOT EXISTS window_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            width REAL NOT NULL DEFAULT 800,
            height REAL NOT NULL DEFAULT 600,
            x REAL NOT NULL DEFAULT 100,
            y REAL NOT NULL DEFAULT 100,
            pinned INTEGER NOT NULL DEFAULT 0
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Level 5: Workspaces table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workspaces (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Level 7: Workspace Directories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workspace_dirs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            workspace_id INTEGER NOT NULL,
            path TEXT NOT NULL,
            collapsed INTEGER DEFAULT 1,
            FOREIGN KEY(workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Add collapsed column if missing
    if !column_exists(conn, "workspace_dirs", "collapsed").await {
        conn.execute("ALTER TABLE workspace_dirs ADD COLUMN collapsed INTEGER DEFAULT 1", ())
            .await
            .map_err(|e| format!("Failed to add collapsed to workspace_dirs: {}", e))?;
    }

    // Add workspace_id column to items if missing
    if !column_exists(conn, "items", "workspace_id").await {
        conn.execute("ALTER TABLE items ADD COLUMN workspace_id INTEGER DEFAULT 1", ())
            .await
            .map_err(|e| format!("Failed to add workspace_id: {}", e))?;
    }

    // Create 4 fixed workspaces if they don't exist (IDs 1-4 are protected)
    conn.execute(
        "INSERT OR IGNORE INTO workspaces (id, name) VALUES (1, 'todos')",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT OR IGNORE INTO workspaces (id, name) VALUES (2, 'files')",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT OR IGNORE INTO workspaces (id, name) VALUES (3, 'others')",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT OR IGNORE INTO workspaces (id, name) VALUES (4, 'web-bookmarks')",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Migrate existing items without workspace_id to default workspace
    conn.execute(
        "UPDATE items SET workspace_id = 1 WHERE workspace_id IS NULL",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// Data Backup and Restore
// ============================================================================


/// Backup all data from current database
pub async fn backup_local_data(conn: &Connection) -> Result<BackupData, String> {
    // Backup items
    // Explicitly select columns to ensure order and completeness
    let mut items_stmt = conn.prepare("SELECT id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, workspace_id, url, summary, created_at, updated_at, content_hash, quick_hash, last_known_path, is_dir FROM items").await.map_err(|e| e.to_string())?;
    let mut items_rows = items_stmt.query(()).await.map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    while let Ok(Some(row)) = items_rows.next().await {
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
        let is_dir: i64 = row.get(18).unwrap_or(0);
        
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
    let mut tags_stmt = conn.prepare("SELECT id, name, color, position FROM tags").await.map_err(|e| e.to_string())?;
    let mut tags_rows = tags_stmt.query(()).await.map_err(|e| e.to_string())?;
    let mut tags = Vec::new();
    while let Ok(Some(row)) = tags_rows.next().await {
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
    let mut ws_stmt = conn.prepare("SELECT id, name FROM workspaces").await.map_err(|e| e.to_string())?;
    let mut ws_rows = ws_stmt.query(()).await.map_err(|e| e.to_string())?;
    let mut workspaces = Vec::new();
    while let Ok(Some(row)) = ws_rows.next().await {
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let name: String = row.get(1).map_err(|e| e.to_string())?;
        
        workspaces.push(serde_json::json!({
            "id": id,
            "name": name
        }));
    }

    // Backup item_tags
    let mut item_tags_stmt = conn.prepare("SELECT item_id, tag_id FROM item_tags").await.map_err(|e| e.to_string())?;
    let mut item_tags_rows = item_tags_stmt.query(()).await.map_err(|e| e.to_string())?;
    let mut item_tags = Vec::new();
    while let Ok(Some(row)) = item_tags_rows.next().await {
        let item_id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let tag_id: i64 = row.get(1).map_err(|e| e.to_string())?;
        
        item_tags.push(serde_json::json!({
            "item_id": item_id,
            "tag_id": tag_id
        }));
    }

    // Backup tag_tags
    let mut tag_tags_stmt = conn.prepare("SELECT child_tag_id, parent_tag_id, position FROM tag_tags").await.map_err(|e| e.to_string())?;
    let mut tag_tags_rows = tag_tags_stmt.query(()).await.map_err(|e| e.to_string())?;
    let mut tag_tags = Vec::new();
    while let Ok(Some(row)) = tag_tags_rows.next().await {
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
pub async fn restore_data(conn: &Connection, backup: BackupData) -> Result<(), String> {
    eprintln!("Restoring data: {} items, {} tags, {} workspaces, {} item_tags, {} tag_tags",
              backup.items.len(), backup.tags.len(), backup.workspaces.len(), backup.item_tags.len(), backup.tag_tags.len());
    
    // Restore workspaces first
    for ws in backup.workspaces {
        let id = ws["id"].as_i64().unwrap();
        let name = ws["name"].as_str().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO workspaces (id, name) VALUES (?, ?)",
            libsql::params![id, name]
        ).await.map_err(|e| e.to_string())?;
    }
    
    // Restore tags
    for tag in backup.tags {
        let id = tag["id"].as_i64().unwrap();
        let name = tag["name"].as_str().unwrap();
        let color = tag["color"].as_str(); // Option<String>
        let position = tag["position"].as_i64().unwrap_or(0);
        
        // Handle explicit null for color if needed, params! handles Option nicely usually but let's be explicit if needed
        // libsql params! macro should handle Option<&str> correctly as NULL if None
        conn.execute(
            "INSERT OR REPLACE INTO tags (id, name, color, position) VALUES (?, ?, ?, ?)",
            libsql::params![id, name, color, position]
        ).await.map_err(|e| e.to_string())?;
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
            libsql::params![id, text, completed, item_type, memo, target_count, current_count, parent_id, position, collapsed, workspace_id, url, summary, created_at, updated_at, content_hash, quick_hash, last_known_path, is_dir]
        ).await.map_err(|e| e.to_string())?;
    }

    // Restore item_tags
    for it in backup.item_tags {
        let item_id = it["item_id"].as_i64().unwrap();
        let tag_id = it["tag_id"].as_i64().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO item_tags (item_id, tag_id) VALUES (?, ?)",
            libsql::params![item_id, tag_id]
        ).await.map_err(|e| e.to_string())?;
    }

    // Restore tag_tags
    for tt in backup.tag_tags {
        let child_tag_id = tt["child_tag_id"].as_i64().unwrap();
        let parent_tag_id = tt["parent_tag_id"].as_i64().unwrap();
        let position = tt["position"].as_i64().unwrap_or(0);
        
        conn.execute(
            "INSERT OR REPLACE INTO tag_tags (child_tag_id, parent_tag_id, position) VALUES (?, ?, ?)",
            libsql::params![child_tag_id, parent_tag_id, position]
        ).await.map_err(|e| e.to_string())?;
    }
    
    eprintln!("Data restore complete");
    Ok(())
}

// ============================================================================
// Cloud Sync Management
// ============================================================================

/// Apply migrations directly to the remote database
pub async fn migrate_remote_schema(url: String, token: String) -> Result<(), String> {
    eprintln!("Connecting to remote DB for migration: {}", url);
    let db = Builder::new_remote(url, token)
        .build()
        .await
        .map_err(|e| format!("Remote build failed: {}", e))?;
        
    let conn = db.connect().map_err(|e| format!("Remote connect failed: {}", e))?;
    
    // Remote connections usually don't support foreign_keys pragma the same way or it might be default? 
    // But safely we can try to run migrations.
    run_migrations(&conn).await?;
    
    eprintln!("Remote schema migration complete");
    Ok(())
}

/// Trigger manual sync (wrapper around DbState public API)
pub async fn sync_db(state: &DbState) -> Result<(), String> {
    // Use DbState's public sync method
    state.sync().await
}

