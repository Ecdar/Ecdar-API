use crate::database::database_context::DatabaseContextTrait;
use crate::entities::{model, query};
use crate::EntityContextTrait;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, Set, Unchanged};
use std::sync::Arc;

pub struct ModelContext {
    db_context: Arc<dyn DatabaseContextTrait>,
}

pub trait ModelContextTrait: EntityContextTrait<model::Model> {}

impl ModelContextTrait for ModelContext {}

impl ModelContext {
    pub fn new(db_context: Arc<dyn DatabaseContextTrait>) -> ModelContext {
        ModelContext { db_context }
    }
}

#[async_trait]
impl EntityContextTrait<model::Model> for ModelContext {
    /// Used for creating a model::Model entity
    /// # Example
    /// ```
    /// let model = model::Model {
    ///     id: Default::default(),
    ///     name: "model::Model name".to_owned(),
    ///     components_info: "{}".to_owned().parse().unwrap(),
    ///     owner_id: 1
    /// };
    /// let model_context: ModelContext = ModelContext::new(...);
    /// model_context.create(model);
    /// ```
    async fn create(&self, entity: model::Model) -> Result<model::Model, DbErr> {
        let model = model::ActiveModel {
            id: Default::default(),
            name: Set(entity.name),
            components_info: Set(entity.components_info),
            owner_id: Set(entity.owner_id),
        };
        let model: model::Model = model.insert(&self.db_context.get_connection()).await?;
        Ok(model)
    }

    /// Returns a single model entity (Uses primary key)
    /// # Example
    /// ```
    /// let model_context: ModelContext = ModelContext::new(...);
    /// let model = model_context.get_by_id(1).unwrap();
    /// ```
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<model::Model>, DbErr> {
        model::Entity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    /// Returns a all model entities (Uses primary key)
    /// # Example
    /// ```
    /// let model_context: ModelContext = ModelContext::new(...);
    /// let model = model_context.get_all().unwrap();
    /// ```
    async fn get_all(&self) -> Result<Vec<model::Model>, DbErr> {
        model::Entity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    /// Updates a single model entity
    /// # Example
    /// ```
    /// let update_model = model::Model {
    ///     name: "new name",
    ///     ..original_model
    /// };
    ///
    /// let model_context: ModelContext = ModelContext::new(...);
    /// let model = model_context.update(update_model).unwrap();
    /// ```
    async fn update(&self, entity: model::Model) -> Result<model::Model, DbErr> {
        let existing_model = self.get_by_id(entity.id).await?;

        return match existing_model {
            None => Err(DbErr::RecordNotUpdated),
            Some(existing_model) => {
                let queries: Vec<query::Model> = existing_model
                    .find_related(query::Entity)
                    .all(&self.db_context.get_connection())
                    .await?;
                for q in queries.iter() {
                    let mut aq = q.clone().into_active_model();
                    aq.outdated = Set(true);
                    aq.update(&self.db_context.get_connection()).await?;
                }
                model::ActiveModel {
                    id: Unchanged(entity.id),
                    name: Set(entity.name),
                    components_info: Set(entity.components_info),
                    owner_id: Unchanged(entity.id),
                }
                .update(&self.db_context.get_connection())
                .await
            }
        };
    }

    /// Returns and deletes a single model entity
    /// # Example
    /// ```
    /// let model_context: ModelContext = ModelContext::new(...);
    /// let model = model_context.delete().unwrap();
    /// ```
    async fn delete(&self, entity_id: i32) -> Result<model::Model, DbErr> {
        let model = self.get_by_id(entity_id).await?;
        match model {
            None => Err(DbErr::RecordNotFound("No record was deleted".into())),
            Some(model) => {
                model::Entity::delete_by_id(entity_id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(model)
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/database/model_context.rs"]
mod model_context_tests;
