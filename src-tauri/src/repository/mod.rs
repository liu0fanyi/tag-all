//! Repository Layer
//!
//! Data access implementations.

pub mod db;
pub mod traits;
pub mod item_repo;
pub mod tag_repo;
pub mod window_state_repo;
pub mod workspace_repo;

#[cfg(test)]
mod tests;

pub use item_repo::ItemRepository;
pub use tag_repo::TagRepository;
pub use window_state_repo::{WindowStateRepository, WindowState};
pub use workspace_repo::WorkspaceRepository;
pub use traits::{Repository, HierarchyRepository, SearchableRepository};
pub use db::{init_db, DbState};
