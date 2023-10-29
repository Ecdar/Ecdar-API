use crate::database::database_context::DatabaseContextTrait;
use crate::database::entity_context::EntityContextTrait;
use crate::entities::prelude::Query as QueryEntity;
use crate::entities::query::{ActiveModel, Model as Query};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

pub struct QueryContext {
    db_context: Box<dyn DatabaseContextTrait>,
}

pub trait QueryContextTrait: EntityContextTrait<Query> {}

impl QueryContextTrait for QueryContext {}

#[async_trait]
impl EntityContextTrait<Query> for QueryContext {
    fn new(db_context: Box<dyn DatabaseContextTrait>) -> QueryContext {
        QueryContext { db_context }
    }

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
    async fn create(&self, entity: Query) -> Result<Query, DbErr> {
        let query = ActiveModel {
            id: Default::default(),
            string: Set(entity.string),
            model_id: Set(entity.model_id),
            result: Set(entity.result),
            outdated: Set(entity.outdated),
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
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<Query>, DbErr> {
        QueryEntity::find_by_id(entity_id)
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
    async fn get_all(&self) -> Result<Vec<Query>, DbErr> {
        QueryEntity::find()
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
    async fn update(&self, entity: Query) -> Result<Query, DbErr> {
        ActiveModel {
            id: Unchanged(entity.id),
            string: Set(entity.string),
            result: Set(entity.result),
            outdated: Set(entity.outdated),
            model_id: Set(entity.model_id),
        }
        .update(&self.db_context.get_connection())
        .await
    }

    /// Delete a query entity by id
    async fn delete(&self, entity_id: i32) -> Result<Query, DbErr> {
        let query = self.get_by_id(entity_id).await?;
        match query {
            None => Err(DbErr::RecordNotFound("No record was deleted".into())),
            Some(query) => {
                QueryEntity::delete_by_id(entity_id)
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
