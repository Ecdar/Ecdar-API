#[cfg(test)]
mod database_tests {
    use crate::tests::database::helpers::{
        create_models, create_queries, create_users, setup_db_with_entities, AnyEntity,
    };
    use crate::{
        database::{entity_context::EntityContextTrait, query_context::QueryContext},
        entities::{model, query, user},
        to_active_models,
    };
    use sea_orm::{entity::prelude::*, IntoActiveModel};

    async fn seed_db() -> (QueryContext, query::Model, model::Model) {
        let db_context =
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model, AnyEntity::Query]).await;

        let query_context = QueryContext::new(db_context.clone());

        let user = create_users(1)[0].clone();
        let model = create_models(1, user.id)[0].clone();
        let query = create_queries(1, model.id)[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();
        model::Entity::insert(model.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        (query_context, query, model)
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

        let fetched_query = query::Entity::find_by_id(query.model_id)
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
            .get_by_id(query.model_id)
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
        let (query_context, _, model) = seed_db().await;

        let queries = create_queries(10, model.id);

        query::Entity::insert_many(to_active_models!(queries.clone()))
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(query_context.get_all().await.unwrap().len(), 10);

        let mut sorted = queries.clone();
        sorted.sort_by_key(|k| k.model_id);

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

        let fetched_query = query::Entity::find_by_id(updated_query.model_id)
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
            string: query.clone().string + "123".into(),
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
    async fn update_does_not_modify_model_id_test() {
        let (query_context, query, _) = seed_db().await;

        query::Entity::insert(query.clone().into_active_model())
            .exec(&query_context.db_context.get_connection())
            .await
            .unwrap();

        let new_query = query::Model {
            model_id: query.model_id + 1,
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

        let deleted_query = query_context.delete(query.model_id).await.unwrap();

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
