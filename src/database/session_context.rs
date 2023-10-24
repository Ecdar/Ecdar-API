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
        SessionContext {
            db_context: db_context,
        }
    }

    async fn create(&self, entity: Model) -> Result<Model, DbErr> {
        let session = ActiveModel {
            id: Default::default(),
            token: Set(entity.token),
            created_at: Set(entity.created_at),
            user_id: Set(entity.user_id),
        };

        let session = session.insert(&self.db_context.db).await;
        session
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

    async fn delete(&self, id: i32) -> Result<Model, DbErr> {
        todo!()
    }
}

#[cfg(test)]
#[path = "../tests/database/session_context.rs"]
mod tests;

