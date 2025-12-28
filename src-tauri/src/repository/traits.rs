//! Repository Layer - Core Traits
//!
//! Defines the abstract interfaces for data access.
//! Implementations can use SQLite, in-memory, etc.

use async_trait::async_trait;
use crate::domain::{Entity, DomainResult};

/// Core repository trait for CRUD operations
///
/// Generic over any Entity type.
/// All operations are async to support various backends.
#[async_trait]
pub trait Repository<T: Entity>: Send + Sync {
    /// Create a new entity
    async fn create(&self, entity: &T) -> DomainResult<T>;

    /// Find entity by ID
    async fn find_by_id(&self, id: T::Id) -> DomainResult<Option<T>>;

    /// List all entities
    async fn list(&self) -> DomainResult<Vec<T>>;

    /// Update an existing entity
    async fn update(&self, entity: &T) -> DomainResult<T>;

    /// Delete entity by ID
    async fn delete(&self, id: T::Id) -> DomainResult<()>;
}

/// Extension for repositories that support hierarchical structure (Level 2)
#[async_trait]
pub trait HierarchyRepository<T: Entity>: Repository<T> {
    /// Get children of a parent (None = get root items)
    async fn get_children(&self, parent_id: Option<T::Id>) -> DomainResult<Vec<T>>;
    
    /// Move item to new parent at specified position
    async fn move_to(&self, id: T::Id, new_parent_id: Option<T::Id>, position: i32) -> DomainResult<()>;
    
    /// Get all descendants of an item (recursive)
    async fn get_descendants(&self, id: T::Id) -> DomainResult<Vec<T>>;
    
    /// Toggle collapsed state
    async fn toggle_collapsed(&self, id: T::Id) -> DomainResult<bool>;
}

/// Extension for repositories that support text search
#[async_trait]
pub trait SearchableRepository<T: Entity>: Repository<T> {
    /// Search entities by text query
    async fn search(&self, query: &str) -> DomainResult<Vec<T>>;
}
