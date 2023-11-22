use crate::database::database_context::DatabaseContextTrait;
use crate::entities::in_use;
use crate::EntityContextTrait;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, Set, Unchanged};
use std::sync::Arc;

pub struct InUseContext {
    db_context: Arc<dyn DatabaseContextTrait>,
}

pub trait InUseContextTrait: EntityContextTrait<in_use::Model> {}

impl InUseContextTrait for InUseContext {}

impl InUseContext {
    pub fn new(db_context: Arc<dyn DatabaseContextTrait>) -> InUseContext {
        InUseContext { db_context }
    }
}
#[async_trait]
impl EntityContextTrait<in_use::Model> for InUseContext {
    /// Used for creating a Model entity
    async fn create(&self, entity: in_use::Model) -> Result<in_use::Model, DbErr> {
        let in_use = in_use::ActiveModel {
            model_id: Set(entity.model_id),
            session_id: Set(entity.session_id),
            latest_activity: Set(Utc::now().naive_local()),
        };
        let in_use: in_use::Model = in_use.insert(&self.db_context.get_connection()).await?;
        Ok(in_use)
    }

    async fn get_by_id(&self, entity_id: i32) -> Result<Option<in_use::Model>, DbErr> {
        in_use::Entity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    async fn get_all(&self) -> Result<Vec<in_use::Model>, DbErr> {
        in_use::Entity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    async fn update(&self, entity: in_use::Model) -> Result<in_use::Model, DbErr> {
        in_use::ActiveModel {
            model_id: Unchanged(entity.model_id),
            session_id: Unchanged(entity.session_id),
            latest_activity: Set(entity.latest_activity),
        }
        .update(&self.db_context.get_connection())
        .await
    }

    async fn delete(&self, entity_id: i32) -> Result<in_use::Model, DbErr> {
        let in_use = self.get_by_id(entity_id).await?;
        match in_use {
            None => Err(DbErr::RecordNotFound("No record was deleted".into())),
            Some(in_use) => {
                in_use::Entity::delete_by_id(entity_id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(in_use)
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/database/in_use_context.rs"]
mod tests;
