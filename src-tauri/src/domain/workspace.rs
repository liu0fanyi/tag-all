//! Workspace domain entity

use serde::{Deserialize, Serialize};
use super::entity::Entity;

/// Workspace represents an isolated data space for items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: u32,
    pub name: String,
}

impl Entity for Workspace {
    type Id = u32;
    
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Workspace {
    pub fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }
}
