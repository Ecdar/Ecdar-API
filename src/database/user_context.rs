use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use std::fmt::Error;
use std::process::id;

use crate::database::database_context::DatabaseContext;
use crate::database::entity_context::EntityContextTrait;
use crate::entities::prelude::User;
use crate::entities::user::{ActiveModel, Model};

pub struct UserContext {
    db_context: DatabaseContext,
}

#[async_trait]
pub trait UserContextTrait {}

impl UserContextTrait for UserContext {}

#[async_trait]
impl EntityContextTrait<Model> for UserContext {
    fn new(db_context: DatabaseContext) -> Self {
        UserContext { db_context }
    }

    async fn create(&self, entity: Model) -> Result<Model, DbErr> {
        let user = ActiveModel {
            id: Default::default(),
            email: Set(entity.email),
            username: Set(entity.username),
            password: Set(entity.password),
        };

        let user: Model = user.insert(&self.db_context.db).await?;
        Ok(user)
    }

    async fn get_by_id(&self, _id: i32) -> Result<Option<Model>, DbErr> {

         User::find_by_id(_id).one(&self.db_context.db).await
    }

    async fn get_all(&self) -> Result<Vec<Model>, DbErr> {
        todo!()
    }

    async fn update(&self, _entity: Model) -> Result<Model, DbErr> {
        todo!()
    }

    async fn delete(&self, _entity: Model) -> Result<Model, DbErr> {
        todo!()
    }
}
#[cfg(test)]
#[path="../tests/database/user_context.rs"]
mod tests;
