use crate::database::database_context::DatabaseContextTrait;
use crate::entities::in_use;
use crate::EntityContextTrait;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, Set, Unchanged};

pub struct InUseContext {
    db_context: Box<dyn DatabaseContextTrait>,
}

pub trait InUseContextTrait: EntityContextTrait<in_use::Model> {}

impl InUseContextTrait for InUseContext {}

#[async_trait]
impl EntityContextTrait<in_use::Model> for InUseContext {
    fn new(db_context: Box<dyn DatabaseContextTrait>) -> InUseContext {
        InUseContext { db_context }
    }

    /// Used for creating an in_use::Model entity
    /// # Example
    /// Assuming you have a `model` variable of type model::Model and `session` variable of type session::Model.
    /// ```rust
    /// let in_use = in_use::Model {
    ///     model_id: model.id,
    ///     session_id: session.id,
    ///     latest_activity: Utc::now().naive_local(),
    /// };
    /// let in_use_context: InUseContext = InUseContext::new(...);
    /// in_use_context.create(in_use);
    /// ```
    async fn create(&self, entity: in_use::Model) -> Result<in_use::Model, DbErr> {
        let in_use = in_use::ActiveModel {
            model_id: Set(entity.model_id),
            session_id: Set(entity.session_id),
            latest_activity: Set(Utc::now().naive_local()),
        };
        let in_use: in_use::Model = in_use.insert(&self.db_context.get_connection()).await?;
        Ok(in_use)
    }

    /// Returns a single in_use entity (Uses a model id as key)
    /// # Example
    /// Assuming you have a `model` variable of type model::Model.
    /// ```rust
    /// let in_use_context: InUseContext = InUseContext::new(...);
    /// let in_use = in_use_context.get_by_id(model.id).unwrap();
    /// ```
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<in_use::Model>, DbErr> {
        in_use::Entity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    /// Returns a all in_use entities
    /// # Example
    /// ```rust
    /// let in_use_context: InUseContext = InUseContext::new(...);
    /// let in_use = in_use_context.get_all().unwrap();
    /// ```
    async fn get_all(&self) -> Result<Vec<in_use::Model>, DbErr> {
        in_use::Entity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    /// Updates a single in_use entity
    /// # Example
    /// ```rust
    /// let update_in_use = in_use::Model {
    ///     latest_activity: Utc::now().naive_local(),
    ///     ..original_in_use
    /// };
    ///
    /// let model_context: ModelContext = ModelContext::new(...);
    /// let model = model_context.update(update_in_use).unwrap();
    /// ```
    async fn update(&self, entity: in_use::Model) -> Result<in_use::Model, DbErr> {
        in_use::ActiveModel {
            model_id: Unchanged(entity.model_id),
            session_id: Unchanged(entity.session_id),
            latest_activity: Set(entity.latest_activity),
        }
        .update(&self.db_context.get_connection())
        .await
    }

    /// Returns and deletes a single in_use entity
    /// # Example
    /// Assuming that `id` is a variable containing the id of the entity to be deleted.
    /// ```rust
    /// let in_use_context: InUseContext = InUseContext::new(...);
    /// let in_use = in_use_context.delete(id).unwrap();
    /// ```
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

