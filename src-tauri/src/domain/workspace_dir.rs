//! Workspace Directory entity
//! 
//! Represents a directory mounted into a workspace.

use serde::{Deserialize, Serialize};
use super::entity::Entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDir {
    pub id: u32,
    pub workspace_id: u32,
    pub path: String,
    // collapsed state: default true (hidden)
    #[serde(default = "default_collapsed")]
    pub collapsed: bool,
}

fn default_collapsed() -> bool {
    true
}

impl Entity for WorkspaceDir {
    type Id = u32;
    
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl WorkspaceDir {
    pub fn new(id: u32, workspace_id: u32, path: String) -> Self {
        Self { id, workspace_id, path, collapsed: true }
    }
}
