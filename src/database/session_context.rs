use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use crate::database::database_context::DatabaseContext;
use crate::database::entity_context::EntityContextTrait;
use crate::entities::prelude::Session;
use crate::entities::session::{ActiveModel, Model};

pub struct SessionContext {
    db_context: DatabaseContext,
}

#[async_trait]
pub trait SessionContextTrait {}

impl SessionContextTrait for SessionContext {}

#[async_trait]
impl EntityContextTrait<Model> for SessionContext {
    fn new(db_context: DatabaseContext) -> Self {
        todo!()
    }

    async fn create(&self, entity: Model) -> Result<Model, DbErr> {
        todo!()
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<Model>, DbErr> {
        todo!()
    }

    async fn get_all(&self) -> Result<Vec<Model>, DbErr> {
        todo!()
    }

    async fn update(&self, entity: Model) -> Result<Model, DbErr> {
        todo!()
    }

    async fn delete(&self, entity: Model) -> Result<Model, DbErr> {
        todo!()
    }
}

#[cfg(test)]
#[path = "../tests/database/session_context.rs"]
mod tests;

