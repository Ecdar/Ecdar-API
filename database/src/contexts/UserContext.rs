use crate::contexts::EntityContext::EntityContextTrait;
use crate::DatabaseContext;
use entities::entities::user::{Model, ActiveModel};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DbErr};
use sea_orm::ActiveValue::{Set, NotSet, Unchanged};
use std::fmt::Error;

#[async_trait]
pub trait UserContextTrait {}

pub struct UserContext {
    db_context: DatabaseContext,
}

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

    async fn get_by_id(&self, id: i32) -> Result<Option<Model>, Error> {
        todo!()
    }

    async fn get_all(&self) -> Result<Vec<Model>, Error> {
        todo!()
    }

    async fn update(&self, entity: Model) -> Result<Model, Error> {
        todo!()
    }

    async fn delete(&self, entity: Model) -> Result<Model, Error> {
        todo!()
    }
}
