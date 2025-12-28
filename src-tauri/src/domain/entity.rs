//! Domain Layer - Core Entity Trait
//!
//! This trait defines the basic contract for all domain entities.
//! All entities must have a unique ID and be thread-safe.

use serde::{Deserialize, Serialize};

/// Core trait for all domain entities
pub trait Entity: Sized + Send + Sync + Clone {
    /// The type of the entity's unique identifier
    type Id: Copy + Eq + std::hash::Hash + Send + Sync;

    /// Returns the entity's unique identifier
    fn id(&self) -> Self::Id;
}

/// Common result type for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

/// Domain-level errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainError {
    NotFound(String),
    InvalidInput(String),
    Conflict(String),
    Internal(String),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DomainError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            DomainError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            DomainError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}
