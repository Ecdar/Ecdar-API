use std::fmt::{Debug, Formatter};

use sea_orm::prelude::async_trait::async_trait;
use sea_orm::DbErr;

use crate::database::database_context::DatabaseContextTrait;

#[async_trait]
pub trait EntityContextTrait<T> {
    fn new(db_context: Box<dyn DatabaseContextTrait>) -> Self
    where
        Self: Sized;
    async fn create(&self, entity: T) -> Result<T, DbErr>;
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<T>, DbErr>;
    async fn get_all(&self) -> Result<Vec<T>, DbErr>;
    async fn update(&self, entity: T) -> Result<T, DbErr>;
    async fn delete(&self, entity_id: i32) -> Result<T, DbErr>;
}

impl<T> Debug for dyn EntityContextTrait<T> + Send + Sync + 'static {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityContextTrait").finish()
    }
}
