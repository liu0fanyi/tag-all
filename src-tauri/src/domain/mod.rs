//! Domain Layer
//!
//! Contains all domain entities and core abstractions.
//! This layer has NO external dependencies (except serde for serialization).

mod entity;
mod item;
mod tag;
mod workspace;
mod workspace_dir;
mod file_id;

pub use entity::{Entity, DomainError, DomainResult};
pub use item::{Item, ItemType};
pub use tag::{Tag};
pub use workspace::Workspace;
pub use workspace_dir::WorkspaceDir;
pub use file_id::FileIdentifier;
