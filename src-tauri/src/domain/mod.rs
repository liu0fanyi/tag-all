//! Domain Layer
//!
//! Contains all domain entities and core abstractions.
//! This layer has NO external dependencies (except serde for serialization).

mod entity;
mod item;

pub use entity::{Entity, DomainError, DomainResult};
pub use item::{Item, ItemType};
