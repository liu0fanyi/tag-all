//! Window State Repository
//!
//! Manages window position/size persistence.

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
    pub pinned: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            x: 100.0,
            y: 100.0,
            pinned: false,
        }
    }
}

pub struct WindowStateRepository {
    // Matches generic DbState
    conn: Arc<Mutex<Option<Connection>>>,
}

impl WindowStateRepository {
    pub fn new(conn: Arc<Mutex<Option<Connection>>>) -> Self {
        Self { conn }
    }

    pub async fn save(&self, state: &WindowState) -> Result<(), String> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or("Database connection not initialized")?;
        
        let now = chrono::Local::now().timestamp_millis();
        conn.execute(
            "INSERT OR REPLACE INTO window_state (id, width, height, x, y, pinned, updated_at) VALUES (1, ?, ?, ?, ?, ?, ?)",
            params![state.width, state.height, state.x, state.y, if state.pinned { 1 } else { 0 }, now],
        )
        .map_err(|e| e.to_string())?;
        
        Ok(())
    }

    pub async fn load(&self) -> Result<Option<WindowState>, String> {
        let guard = self.conn.lock().await;
        let conn = guard.as_ref().ok_or("Database connection not initialized")?;
        
        let mut stmt = conn.prepare("SELECT width, height, x, y, pinned FROM window_state WHERE id = 1")
            .map_err(|e| e.to_string())?;
            
        let mut rows = stmt.query([])
            .map_err(|e| e.to_string())?;
        
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(WindowState {
                width: row.get::<_, f64>(0).unwrap_or(800.0),
                height: row.get::<_, f64>(1).unwrap_or(600.0),
                x: row.get::<_, f64>(2).unwrap_or(100.0),
                y: row.get::<_, f64>(3).unwrap_or(100.0),
                pinned: row.get::<_, i32>(4).unwrap_or(0) != 0,
            }))
        } else {
            Ok(None)
        }
    }
}
