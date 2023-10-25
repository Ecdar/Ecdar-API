use std::fmt::Display;
use crate::entities::model::{Model, ActiveModel};
use crate::entities::prelude::Model as ModelEntity;
use crate::database::database_context::DatabaseContextTrait;
use crate::EntityContextTrait;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, RuntimeErr, Set, Unchanged};

pub struct ModelContext {
    db_context: Box<dyn DatabaseContextTrait>,
}

pub trait ModelContextTrait: EntityContextTrait<Model> {}

impl ModelContextTrait for ModelContext {}

#[async_trait]
impl EntityContextTrait<Model> for ModelContext {
    fn new(db_context: Box<dyn DatabaseContextTrait>) -> ModelContext {
        ModelContext { db_context }
    }

    /// Used for creating a Model entity
    /// # Example 
    /// ```
    /// let model = Model {
    ///     id: Default::default(),
    ///     name: "Model name".to_owned(),
    ///     components_info: "{}".to_owned().parse().unwrap(),
    ///     owner_id: 1
    /// };
    /// let model_context: ModelContext = ModelContext::new(...);
    /// model_context.create(model);
    /// ```
    async fn create(&self, entity: Model) -> Result<Model, DbErr> {
        let model = ActiveModel {
            id: Default::default(),
            name: Set(entity.name),
            components_info: Set(entity.components_info),
            owner_id: Set(entity.owner_id),
        };
        let model: Model = model.insert(&self.db_context.get_connection()).await?;
        Ok(model)
    }

    /// Returns a single model entity (Uses primary key)
    /// # Example
    /// ```
    /// let model_context: ModelContext = ModelContext::new(...);
    /// let model = model_context.get_by_id(1).unwrap();
    /// ```
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<Model>, DbErr> {
        ModelEntity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    /// Returns a all model entities (Uses primary key)
    /// # Example
    /// ```
    /// let model_context: ModelContext = ModelContext::new(...);
    /// let model = model_context.get_all().unwrap();
    /// ```
    async fn get_all(&self) -> Result<Vec<Model>, DbErr> {
        ModelEntity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    /// Updates a single model entity
    /// # Example
    /// ```
    /// let update_model = Model {
    ///     name: "new name",
    ///     ..original_model
    /// };
    /// 
    /// let model_context: ModelContext = ModelContext::new(...);
    /// let model = model_context.update(update_model).unwrap();
    /// ```
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

    /// Returns and deletes a single model entity
    /// # Example
    /// ```
    /// let model_context: ModelContext = ModelContext::new(...);
    /// let model = model_context.delete().unwrap();
    /// ```
    async fn delete(&self, entity_id: i32) -> Result<Model, DbErr> {
        let model = self.get_by_id(entity_id).await?;
        match model {
            None => Err(DbErr::Exec(RuntimeErr::Internal(
                "No record was deleted".into(),
            ))),
            Some(model) => {
                ModelEntity::delete_by_id(entity_id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(model)
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/database/model_context.rs"]
mod tests;
