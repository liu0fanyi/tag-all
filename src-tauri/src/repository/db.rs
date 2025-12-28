//! Database Connection and Setup
//!
//! Manages SQLite database connection and migrations.

use libsql::{Builder, Connection, Database};
use std::path::PathBuf;
use tokio::sync::Mutex;

/// Database state wrapper
pub struct DbState {
    db: Mutex<Option<Database>>,
    conn: Mutex<Option<Connection>>,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            db: Mutex::new(None),
            conn: Mutex::new(None),
        }
    }

    /// Get a connection, initializing if necessary
    pub async fn get_connection(&self) -> Result<Connection, String> {
        let guard = self.conn.lock().await;
        if let Some(conn) = &*guard {
            return Ok(conn.clone());
        }
        Err("Database not initialized".to_string())
    }
}

/// Initialize database with path
pub async fn init_db(db_path: &PathBuf) -> Result<DbState, String> {
    let db_path_str = db_path.to_str().ok_or("Invalid DB path")?;

    let db = Builder::new_local(db_path_str)
        .build()
        .await
        .map_err(|e| format!("Failed to build db: {}", e))?;

    let conn = db.connect().map_err(|e| format!("Failed to connect: {}", e))?;

    // Run migrations
    run_migrations(&conn).await?;

    let state = DbState::new();
    *state.db.lock().await = Some(db);
    *state.conn.lock().await = Some(conn);

    Ok(state)
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

    // Create index for faster parent-child queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_items_parent ON items(parent_id)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Level 5 will add: workspace_id column

    Ok(())
}
