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

// Re-export database types and functions (including shared crate functions)
pub use db::{init_db, SyncConfig, BackupData, sync_db, DbState, configure_sync, get_sync_config};
