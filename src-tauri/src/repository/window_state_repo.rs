//! Window State Repository
//!
//! Manages window position/size persistence.

use libsql::Connection;
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
    conn: Arc<Mutex<Connection>>,
}

impl WindowStateRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub async fn save(&self, state: &WindowState) -> Result<(), String> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT OR REPLACE INTO window_state (id, width, height, x, y, pinned) VALUES (1, ?, ?, ?, ?, ?)",
            libsql::params![state.width, state.height, state.x, state.y, if state.pinned { 1 } else { 0 }],
        )
        .await
        .map_err(|e| e.to_string())?;
        
        Ok(())
    }

    pub async fn load(&self) -> Result<Option<WindowState>, String> {
        let conn = self.conn.lock().await;
        
        let mut rows = conn
            .query("SELECT width, height, x, y, pinned FROM window_state WHERE id = 1", ())
            .await
            .map_err(|e| e.to_string())?;
        
        if let Ok(Some(row)) = rows.next().await {
            Ok(Some(WindowState {
                width: row.get::<f64>(0).unwrap_or(800.0),
                height: row.get::<f64>(1).unwrap_or(600.0),
                x: row.get::<f64>(2).unwrap_or(100.0),
                y: row.get::<f64>(3).unwrap_or(100.0),
                pinned: row.get::<i32>(4).unwrap_or(0) != 0,
            }))
        } else {
            Ok(None)
        }
    }
}
