//! The base trait for all database entities. Exposes basic CRUD functionality for.
//! Some specific entities might need additional functionality, but that should implemented in entity-specific traits.
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::DbErr;

#[async_trait]
/// The base trait for all database entities. Exposes basic CRUD functionality for.
/// Some specific entities might need additional functionality, but that should implemented in entity-specific traits.
pub trait EntityContextTrait<T>: Send + Sync {
    /// Inserts an entity into the database
    /// # Errors
    /// Errors on failed connection, execution error or constraint violations.
    /// # Notes
    /// Most implementations does not allow the caller to set the primary key manually, 
    /// if the key is needed, use the returned value to ensure that the correct key is used
    async fn create(&self, entity: T) -> Result<T, DbErr>;
    /// Searches for an entity by its primary key, returning [`Some`] if an entity is found, [`None`] otherwise
    /// # Errors
    /// Errors on failed connection, execution error or constraint violations.
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<T>, DbErr>;
    /// Returns all the entities in the table
    /// # Errors
    /// Errors on failed connection, execution error or constraint violations.
    async fn get_all(&self) -> Result<Vec<T>, DbErr>;
    /// Updates a given entity. This is usually done by searching by primary key
    /// # Errors
    /// Errors on failed connection, execution error or constraint violations.
    /// # Notes 
    /// It is not possible to change the primary key, as it is used to look up the given entity.
    async fn update(&self, entity: T) -> Result<T, DbErr>;
    /// Searches for an entity by primary key and deletes it
    /// # Errors
    /// Errors on failed connection, execution error or constraint violations.
    async fn delete(&self, entity_id: i32) -> Result<T, DbErr>;
}
