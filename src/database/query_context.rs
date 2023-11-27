use crate::database::database_context::DatabaseContextTrait;
use crate::database::entity_context::EntityContextTrait;
use crate::entities::query;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, NotSet, QueryFilter};
use std::sync::Arc;

pub struct QueryContext {
    db_context: Arc<dyn DatabaseContextTrait>,
}

#[async_trait]
pub trait QueryContextTrait: EntityContextTrait<query::Model> {
    async fn get_all_by_model_id(&self, model_id: i32) -> Result<Vec<query::Model>, DbErr>;
}

#[async_trait]
impl QueryContextTrait for QueryContext {
    async fn get_all_by_model_id(&self, model_id: i32) -> Result<Vec<query::Model>, DbErr> {
        query::Entity::find()
            .filter(query::Column::ModelId.eq(model_id))
            .all(&self.db_context.get_connection())
            .await
    }
}

impl QueryContext {
    pub fn new(db_context: Arc<dyn DatabaseContextTrait>) -> QueryContext {
        QueryContext { db_context }
    }
}

#[async_trait]
impl EntityContextTrait<query::Model> for QueryContext {
    /// Used for creating a query entity
    /// ## Example
    /// ```
    /// let model : Model = {
    ///     id: Default::default(),
    ///     string: "query_string".into(),
    ///     model_id: 1,
    ///     result: "query_result".into(),
    ///     out_dated: true
    /// }
    /// let context : QueryContext = QueryContext::new(...);
    /// context.create(model);
    /// ```
    async fn create(&self, entity: query::Model) -> Result<query::Model, DbErr> {
        let query = query::ActiveModel {
            id: Default::default(),
            string: Set(entity.string),
            model_id: Set(entity.model_id),
            result: NotSet,
            outdated: NotSet,
        };
        let query = query.insert(&self.db_context.get_connection()).await?;
        Ok(query)
    }

    /// Returns a single query entity (uses primary key)
    /// ## Example
    /// ```
    /// let context : QueryContext = QueryContext::new(...);
    /// let model : Model = context.get_by_id(1).unwrap();
    /// assert_eq!(model.string,"query_string".into());
    /// ```
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<query::Model>, DbErr> {
        query::Entity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    /// Returns all the query entities
    /// ## Example
    /// ```
    /// let context : QueryContext = QueryContext::new(...);
    /// let model : vec<Model> = context.get_all().unwrap();
    /// assert_eq!(model.len(),5);
    /// ```
    async fn get_all(&self) -> Result<Vec<query::Model>, DbErr> {
        query::Entity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    /// Updates and returns the given user entity
    /// ## Example
    /// ```
    /// let context : QueryContext = QueryContext::new(...);
    /// let query = context.get_by_id(1).unwrap();
    /// let updated_query = Model {
    ///     id: query.id,
    ///     string: query.string,
    ///     model_id: query.model_id,
    ///     result: query.result,
    ///     out_dated: false
    /// }
    /// assert_eq!(context.update(updated_query).unwrap(),Model {
    ///     id: 1,
    ///     string: "query_string".into(),
    ///     model_id: 1,
    ///     result: "query_result".into(),
    ///     out_dated: false
    /// }
    /// ```
    /// ## Note
    /// The user entity's id will never be changed. If this behavior is wanted, delete the old user and create a one.
    async fn update(&self, entity: query::Model) -> Result<query::Model, DbErr> {
        query::ActiveModel {
            id: Unchanged(entity.id),
            string: Set(entity.string),
            result: Set(entity.result),
            outdated: Set(entity.outdated),
            model_id: Unchanged(entity.model_id),
        }
        .update(&self.db_context.get_connection())
        .await
    }

    /// Delete a query entity by id
    async fn delete(&self, entity_id: i32) -> Result<query::Model, DbErr> {
        let query = self.get_by_id(entity_id).await?;
        match query {
            None => Err(DbErr::RecordNotFound("No record was deleted".into())),
            Some(query) => {
                query::Entity::delete_by_id(entity_id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(query)
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/database/query_context.rs"]
mod tests;
