use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DbErr};
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait DatabaseContextTrait: Send + Sync + Debug {
    /// Resets the database, usually by truncating all tables, useful for testing on a single database
    /// # Errors
    /// Errors on failed connection or execution error.
    async fn reset(&self) -> Result<Arc<dyn DatabaseContextTrait>, DbErr>;
    /// Gets the connection to the database
    fn get_connection(&self) -> DatabaseConnection;
}
