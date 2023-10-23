
use async_trait::async_trait;
use sea_orm::DbErr;

use crate::entities::model::Model;

use super::{database_context::DatabaseContext, entity_context::EntityContextTrait};

pub struct ModelContext {
    db_context: DatabaseContext,
}

#[async_trait]
pub trait ModelContextTrait {}

impl ModelContextTrait for ModelContext {}

#[async_trait]
impl EntityContextTrait<Model> for ModelContext {
    fn new(db_context: DatabaseContext) -> Self {
        ModelContext { db_context }
    }

    async fn create(&self, entity: Model) -> Result<Model, DbErr> {
        todo!()
    }

    async fn get_by_id(&self, entity_id: i32) -> Result<Option<Model>, DbErr> {
        todo!()
    }

    async fn get_all(&self) -> Result<Vec<Model>, DbErr> {
        todo!()
    }

    async fn update(&self, entity: Model) -> Result<Model, DbErr> {
        todo!()
    }

    async fn delete(&self, entity_id: i32) -> Result<Model, DbErr> {
        todo!()
    }
}
