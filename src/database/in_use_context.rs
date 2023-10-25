use crate::database::database_context::DatabaseContextTrait;
use crate::entities::in_use::{ActiveModel, Model as InUse};
use crate::entities::prelude::InUse as InUseEntity;
use crate::EntityContextTrait;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, RuntimeErr, Set, Unchanged};

pub struct InUseContext {
    db_context: Box<dyn DatabaseContextTrait>,
}

pub trait InUseContextTrait: EntityContextTrait<InUse> {}

impl InUseContextTrait for InUseContext {}

#[async_trait]
impl EntityContextTrait<InUse> for InUseContext {
    fn new(db_context: Box<dyn DatabaseContextTrait>) -> InUseContext {
        InUseContext { db_context }
    }

    /// Used for creating a Model entity
    async fn create(&self, entity: InUse) -> Result<InUse, DbErr> {
        let in_use = ActiveModel {
            model_id: Set(entity.model_id),
            session_id: Set(entity.session_id),
            latest_activity: Set(entity.latest_activity),
        };
        let in_use: InUse = in_use.insert(&self.db_context.get_connection()).await?;
        Ok(in_use)
    }

    async fn get_by_id(&self, entity_id: i32) -> Result<Option<InUse>, DbErr> {
        InUseEntity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    async fn get_all(&self) -> Result<Vec<InUse>, DbErr> {
        InUseEntity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    async fn update(&self, entity: InUse) -> Result<InUse, DbErr> {
        let res = &self.get_by_id(entity.model_id).await?;
        let updated_in_use: Result<InUse, DbErr> = match res {
            None => Err(DbErr::RecordNotFound(String::from(format!(
                "Could not find entity {:?}",
                entity
            )))),
            Some(in_use) => {
                ActiveModel {
                    model_id: Unchanged(in_use.model_id),
                    session_id: Unchanged(in_use.session_id),
                    latest_activity: Set(entity.latest_activity),
                }
                    .update(&self.db_context.get_connection())
                    .await
            }
        };
        updated_in_use
    }

    async fn delete(&self, entity_id: i32) -> Result<InUse, DbErr> {
        let in_use = self.get_by_id(entity_id).await?;
        match in_use {
            None => Err(DbErr::Exec(RuntimeErr::Internal(
                "No record was deleted".into(),
            ))),
            Some(in_use) => {
                InUseEntity::delete_by_id(entity_id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(in_use)
            }
        }
    }
}

//#[cfg(test)]
//#[path = "../tests/database/model_context.rs"]
//mod tests;
