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

/// Tag data structure (matches backend)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub id: u32,
    pub name: String,
    pub color: Option<String>,
    pub position: i32,
}

/// Workspace data structure (matches backend)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceDir {
    pub id: u32,
    pub workspace_id: u32,
    pub path: String,
    #[serde(default = "default_true")]
    pub collapsed: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileViewItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub last_modified: u64,
    pub quick_hash: String,
    pub db_item: Option<Item>,
    #[serde(default)]
    pub tags: Vec<Tag>,
}
