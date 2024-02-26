use crate::contexts::{DatabaseContextTrait, EntityContextTrait};
use crate::entities::query;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, NotSet, QueryFilter};
use std::sync::Arc;

#[async_trait]
pub trait QueryContextTrait: EntityContextTrait<query::Model> {
    /// Returns the queries associated with a given project id
    async fn get_all_by_project_id(&self, project_id: i32) -> Result<Vec<query::Model>, DbErr>;
}

pub struct QueryContext {
    db_context: Arc<dyn DatabaseContextTrait>,
}
#[async_trait]
impl QueryContextTrait for QueryContext {
    async fn get_all_by_project_id(&self, project_id: i32) -> Result<Vec<query::Model>, DbErr> {
        query::Entity::find()
            .filter(query::Column::ProjectId.eq(project_id))
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
    ///     project_id: 1,
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
            project_id: Set(entity.project_id),
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
    ///     project_id: query.project_id,
    ///     result: query.result,
    ///     out_dated: false
    /// }
    /// assert_eq!(context.update(updated_query).unwrap(),Model {
    ///     id: 1,
    ///     string: "query_string".into(),
    ///     project_id: 1,
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
            project_id: Unchanged(entity.project_id),
        }
        .update(&self.db_context.get_connection())
        .await
    }

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
mod tests {
    use super::super::helpers::{
        create_projects, create_queries, create_users, get_reset_database_context,
    };
    use crate::{
        contexts::EntityContextTrait,
        contexts::QueryContext,
        entities::{project, query, user},
        to_active_models,
    };
    use sea_orm::{entity::prelude::*, IntoActiveModel};

    async fn seed_db() -> (QueryContext, query::Model, project::Model) {
        let db_context = get_reset_database_context().await;

        let query_context = QueryContext::new(db_context);

        let user = create_users(1)[0].clone();
        let project = create_projects(1, user.id)[0].clone();
        let query = create_queries(1, project.id)[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();
        project::Entity::insert(project.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        (query_context, query, project)
    }

    #[tokio::test]
    async fn create_test() {
        let (query_context, query, _) = seed_db().await;

        let created_query = query_context.create(query.clone()).await.unwrap();

        let fetched_query = query::Entity::find_by_id(created_query.id)
            .one(&query_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        // Assert if the fetched access is the same as the created access
        assert_eq!(query, created_query);
        assert_eq!(fetched_query, created_query);
    }

    #[tokio::test]
    async fn create_default_outdated_test() {
        let (query_context, query, _) = seed_db().await;

        let _inserted_query = query_context.create(query.clone()).await.unwrap();

        let fetched_query = query::Entity::find_by_id(query.project_id)
            .one(&query_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert!(fetched_query.outdated)
    }

    #[tokio::test]
    async fn create_auto_increment_test() {
        let (query_context, query, _) = seed_db().await;

        let created_query1 = query_context.create(query.clone()).await.unwrap();
        let created_query2 = query_context.create(query.clone()).await.unwrap();

        let fetched_query1 = query::Entity::find_by_id(created_query1.id)
            .one(&query_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        let fetched_query2 = query::Entity::find_by_id(created_query2.id)
            .one(&query_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_ne!(fetched_query1.id, fetched_query2.id);
        assert_ne!(created_query1.id, created_query2.id);
        assert_eq!(created_query1.id, fetched_query1.id);
        assert_eq!(created_query2.id, fetched_query2.id);
    }

    #[tokio::test]
    async fn get_by_id_test() {
        let (query_context, query, _) = seed_db().await;

        query::Entity::insert(query.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        let fetched_in_use = query_context
            .get_by_id(query.project_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(fetched_in_use, query)
    }

    #[tokio::test]
    async fn get_by_non_existing_id_test() {
        let (query_context, _, _) = seed_db().await;

        let query = query_context.get_by_id(1).await;

        assert!(query.unwrap().is_none())
    }

    #[tokio::test]
    async fn get_all_test() {
        let (query_context, _, project) = seed_db().await;

        let queries = create_queries(10, project.id);

        query::Entity::insert_many(to_active_models!(queries.clone()))
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(query_context.get_all().await.unwrap().len(), 10);

        let mut sorted = queries.clone();
        sorted.sort_by_key(|k| k.project_id);

        for (i, query) in sorted.into_iter().enumerate() {
            assert_eq!(query, queries[i]);
        }
    }

    #[tokio::test]
    async fn get_all_empty_test() {
        let (query_context, _, _) = seed_db().await;

        let queries = query_context.get_all().await.unwrap();

        assert_eq!(0, queries.len())
    }

    #[tokio::test]
    async fn update_test() {
        let (query_context, query, _) = seed_db().await;

        query::Entity::insert(query.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        let new_query = query::Model { ..query };

        let updated_query = query_context.update(new_query.clone()).await.unwrap();

        let fetched_query = query::Entity::find_by_id(updated_query.project_id)
            .one(&query_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(new_query, updated_query);
        assert_eq!(updated_query, fetched_query);
    }

    #[tokio::test]
    async fn update_modifies_string_test() {
        let (query_context, query, _) = seed_db().await;

        query::Entity::insert(query.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        let new_query = query::Model {
            string: query.clone().string + "123",
            ..query.clone()
        };

        let updated_query = query_context.update(new_query.clone()).await.unwrap();

        assert_ne!(query, updated_query);
        assert_ne!(query, new_query);
    }

    #[tokio::test]
    async fn update_modifies_outdated_test() {
        let (query_context, query, _) = seed_db().await;

        query::Entity::insert(query.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        let new_query = query::Model {
            outdated: !query.clone().outdated,
            ..query.clone()
        };

        let updated_query = query_context.update(new_query.clone()).await.unwrap();

        assert_ne!(query, updated_query);
        assert_ne!(query, new_query);
    }

    #[tokio::test]
    async fn update_modifies_result_test() {
        let (query_context, mut query, _) = seed_db().await;

        query.result = Some("{}".to_owned().parse().unwrap());

        query::Entity::insert(query.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        let new_query = query::Model {
            result: None,
            ..query.clone()
        };

        let updated_query = query_context.update(new_query.clone()).await.unwrap();

        assert_ne!(query, updated_query);
        assert_ne!(query, new_query);
    }

    #[tokio::test]
    async fn update_does_not_modify_id_test() {
        let (query_context, query, _) = seed_db().await;

        query::Entity::insert(query.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        let new_query = query::Model {
            id: query.id + 1,
            ..query.clone()
        };

        let updated_query = query_context.update(new_query.clone()).await;

        assert!(matches!(
            updated_query.unwrap_err(),
            DbErr::RecordNotUpdated
        ));
    }

    #[tokio::test]
    async fn update_does_not_modify_project_id_test() {
        let (query_context, query, _) = seed_db().await;

        query::Entity::insert(query.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        let new_query = query::Model {
            project_id: query.project_id + 1,
            ..query.clone()
        };

        let updated_query = query_context.update(new_query.clone()).await.unwrap();

        assert_eq!(query, updated_query);
    }

    #[tokio::test]
    async fn update_non_existing_id_test() {
        let (query_context, query, _) = seed_db().await;

        let updated_query = query_context.update(query.clone()).await;

        assert!(matches!(
            updated_query.unwrap_err(),
            DbErr::RecordNotUpdated
        ));
    }

    #[tokio::test]
    async fn delete_test() {
        let (query_context, query, _) = seed_db().await;

        query::Entity::insert(query.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        let deleted_query = query_context.delete(query.project_id).await.unwrap();

        let all_queries = query::Entity::find()
            .all(&query_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(query, deleted_query);
        assert!(all_queries.is_empty());
    }

    #[tokio::test]
    async fn delete_non_existing_id_test() {
        let (query_context, _, _) = seed_db().await;

        let deleted_query = query_context.delete(1).await;

        assert!(matches!(
            deleted_query.unwrap_err(),
            DbErr::RecordNotFound(_)
        ))
    }
}
