//! The trait that allows various context objects to support basic CRUD operations as well as being shared across threads
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::DbErr;

#[async_trait]
pub trait EntityContextTrait<T>: Send + Sync {
    async fn create(&self, entity: T) -> Result<T, DbErr>;
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<T>, DbErr>;
    async fn get_all(&self) -> Result<Vec<T>, DbErr>;
    async fn update(&self, entity: T) -> Result<T, DbErr>;
    async fn delete(&self, entity_id: i32) -> Result<T, DbErr>;
}
