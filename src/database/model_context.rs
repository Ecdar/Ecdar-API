
use async_trait::async_trait;
use sea_orm::{DbErr, Set, ActiveModelTrait, EntityTrait, Unchanged};
use std::fmt::Display;
use crate::database::database_context::DatabaseContextTrait;
use crate::entities::model::{Model, ActiveModel};
use crate::entities::prelude::Model as ModelEntity;
use crate::EntityContextTrait;

pub struct ModelContext<'a> {
    db_context: &'a dyn DatabaseContextTrait,
}

#[async_trait]
pub trait ModelContextTrait<'a>: EntityContextTrait<'a, Model>{
    fn hello(&self) -> u32;
}

#[async_trait]
impl<'a> ModelContextTrait<'a> for ModelContext<'a> {
    fn hello(&self) -> u32 {
        32
    }
}

#[async_trait]
impl<'a> EntityContextTrait<'a, Model> for ModelContext<'a> {
    fn new(db_context: &dyn DatabaseContextTrait) -> ModelContext {
        ModelContext { db_context }
    }

    /// Used for creating a Model entity
    async fn create(&self, entity: Model) -> Result<Model, DbErr> {
        let model = ActiveModel {
            id: Default::default(),
            name: Set(entity.name),
            components_info: Set(entity.components_info),
            owner_id: Set(entity.owner_id),
        };
        let model: Model = model.insert(self.db_context.get_connection()).await?;
        Ok(model)
    }

    async fn get_by_id(&self, entity_id: i32) -> Result<Option<Model>, DbErr> {
        ModelEntity::find_by_id(entity_id).one(&self.db_context.get_connection()).await
    }

    async fn get_all(&self) -> Result<Vec<Model>, DbErr> {
        ModelEntity::find().all(&self.db_context.get_connection()).await
    }

    async fn update(&self, entity: Model) -> Result<Model, DbErr> {
        let res = &self.get_by_id(entity.id).await?;
        let updated_model: Result<Model, DbErr> = match res {
            None => Err(DbErr::RecordNotFound(String::from(format!(
                "Could not find entity {:?}",
                entity
            )))),
            Some(model) => {
                ActiveModel {
                    id: Unchanged(model.id),
                    name: Set(entity.name),
                    components_info: Set(entity.components_info),
                    owner_id: Unchanged(model.id),
                }
                .update(&self.db_context.get_connection())
                .await
            }
        };
        updated_model
    }

    async fn delete(&self, entity_id: i32) -> Result<Model, DbErr> {
        todo!()
    }
}

//#[cfg(test)]
//#[path = "../tests/database/model_context.rs"]
//mod tests;