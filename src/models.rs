//! Frontend Models
//!
//! Data structures matching backend entities.

use serde::{Deserialize, Serialize};

/// Item data structure (matches backend)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub text: String,
    pub completed: bool,
    pub item_type: String,
    pub memo: Option<String>,
    pub target_count: Option<i32>,
    pub current_count: i32,
    pub parent_id: Option<u32>,
    pub position: i32,
    pub collapsed: bool,
}
