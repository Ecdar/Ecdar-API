use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DbErr};
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait DatabaseContextTrait: Send + Sync + Debug {
    async fn reset(&self) -> Result<Arc<dyn DatabaseContextTrait>, DbErr>;
    fn get_connection(&self) -> DatabaseConnection;
}
