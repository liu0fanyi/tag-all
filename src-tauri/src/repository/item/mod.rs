//! Item Repository Module
//!
//! This module provides item repository functionality split into specialized sub-modules:
//! - item_repo: Core CRUD operations
//! - item_hierarchy: Hierarchy operations (children, descendants, move)
//! - item_positioning: Position management
//! - item_workspace: Workspace-specific operations

mod item_repo;
mod item_hierarchy;
mod item_positioning;
mod item_workspace;

pub use item_repo::ItemRepository;

// Re-export all operation traits so they can be used by importing ItemRepository
pub use item_hierarchy::ItemHierarchyOperations;
pub use item_positioning::ItemPositioningOperations;
pub use item_workspace::ItemWorkspaceOperations;
