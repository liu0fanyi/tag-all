//! Tag Repository Module
//!
//! This module provides tag repository functionality split into specialized sub-modules:
//! - tag_repo: Core CRUD operations
//! - item_tag: Item-Tag relationship operations
//! - tag_hierarchy: Tag-Tag relationship operations (parent-child)
//! - tag_positioning: Position management operations

mod tag_repo;
mod item_tag;
mod tag_hierarchy;
mod tag_positioning;

pub use tag_repo::TagRepository;

// Re-export all operation traits so they can be used by importing TagRepository
pub use item_tag::ItemTagOperations;
pub use tag_hierarchy::TagHierarchyOperations;
pub use tag_positioning::TagPositioningOperations;
