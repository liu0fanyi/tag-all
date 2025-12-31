//! Repository Layer
//!
//! Data access implementations.

pub mod db;
pub mod traits;
pub mod item;
pub mod tag;
pub mod window_state_repo;
pub mod workspace_repo;

#[cfg(test)]
mod tests;

pub use item::ItemRepository;
pub use tag::TagRepository;
pub use window_state_repo::{WindowStateRepository, WindowState};
pub use workspace_repo::WorkspaceRepository;
pub use traits::{Repository, HierarchyRepository};
pub use db::init_db;
