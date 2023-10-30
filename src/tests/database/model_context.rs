#[cfg(test)]
mod database_tests {
    use crate::tests::database::helpers::*;
    use sea_orm::error::DbErr;
    use sea_orm::{entity::prelude::*, IntoActiveModel};
    use std::matches;

    use crate::database::database_context::DatabaseContextTrait;
    use crate::{
        database::{entity_context::EntityContextTrait, model_context::ModelContext},
        entities::{access, in_use, model, query, session, user},
    };

    // Test the functionality of the 'create' function, which creates a user in the database
    #[tokio::test]
    async fn create_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::Model, AnyEntity::User]).await;
        let model_context = ModelContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);

        user::Entity::insert(users[0].clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        // Creates a model of the user which will be created
        let models: Vec<model::Model> = create_models(1, 1);
        let new_model = models[0].clone();

        // Creates the user in the database using the 'create' function
        let created_model = model_context.create(new_model.clone()).await.unwrap();

        let fetched_model = model::Entity::find_by_id(created_model.id)
            .one(&model_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        // Assert if the new_user, created_user, and fetched_user are the same
        assert_eq!(new_model, created_model);
        assert_eq!(created_model, fetched_model);
    }

    #[tokio::test]
    async fn create_auto_increment_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::Model, AnyEntity::User]).await;
        let model_context = ModelContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let models: Vec<model::Model> = create_models(2, users[0].clone().id);

        user::Entity::insert(users[0].clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        // Creates the model in the database using the 'create' function
        let created_model1 = model_context.create(models[0].clone()).await.unwrap();
        let created_model2 = model_context.create(models[1].clone()).await.unwrap();

        let fetched_model1 = model::Entity::find_by_id(created_model1.id)
            .one(&model_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        let fetched_model2 = model::Entity::find_by_id(created_model2.id)
            .one(&model_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        // Assert if the new_user, created_user, and fetched_user are the same
        assert_ne!(fetched_model1.id, fetched_model2.id);
        assert_ne!(created_model1.id, created_model2.id);
        assert_eq!(created_model1.id, fetched_model1.id);
        assert_eq!(created_model2.id, fetched_model2.id);
    }

    #[tokio::test]
    async fn get_by_id_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::Model, AnyEntity::User]).await;
        let model_context = ModelContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let models: Vec<model::Model> = create_models(1, users[0].clone().id);
        let new_model = models[0].clone();

        user::Entity::insert(users[0].clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        // Creates the user in the database using the 'create' function
        model::Entity::insert(new_model.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        // Fetches the user created using the 'get_by_id' function
        let fetched_model = model_context
            .get_by_id(new_model.id)
            .await
            .unwrap()
            .unwrap();

        // Assert if the new_user, created_user, and fetched_user are the same
        assert_eq!(new_model, fetched_model);
    }

    #[tokio::test]
    async fn get_by_non_existing_id_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::Model]).await;
        let model_context = ModelContext::new(db_context.clone());

        // Fetches the user created using the 'get_by_id' function
        let fetched_model = model_context.get_by_id(1).await.unwrap();

        assert!(fetched_model.is_none());
    }

    #[tokio::test]
    async fn get_all_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::Model, AnyEntity::User]).await;
        let model_context = ModelContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let models: Vec<model::Model> = create_models(2, users[0].clone().id);

        user::Entity::insert(users[0].clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let active_models_vec = models
            .clone()
            .into_iter()
            .map(|x| x.into_active_model())
            .collect::<Vec<model::ActiveModel>>();

        model::Entity::insert_many(active_models_vec)
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let result = model_context.get_all().await.unwrap();

        assert_eq!(models, result);
    }

    #[tokio::test]
    async fn update_test() -> () {
        // Setting up database and user context
        let db_context =
            setup_db_with_entities(vec![AnyEntity::Model, AnyEntity::User, AnyEntity::Query]).await;
        let model_context = ModelContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let models: Vec<model::Model> = create_models(2, users[0].clone().id);
        let model = models[0].clone();

        user::Entity::insert(users[0].clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        model::Entity::insert(model.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let new_model = model::Model {
            name: models[0].clone().name + "123",
            ..model
        };

        let updated_model = model_context.update(new_model.clone()).await.unwrap();

        let fetched_model = model::Entity::find_by_id(updated_model.id)
            .one(&db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(new_model, updated_model);
        assert_eq!(updated_model, fetched_model);
    }

    #[tokio::test]
    async fn update_check_query_outdated_test() {
        // Setting up database and user context
        let db_context =
            setup_db_with_entities(vec![AnyEntity::Model, AnyEntity::Query, AnyEntity::User]).await;
        let model_context = ModelContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let models: Vec<model::Model> = create_models(2, users[0].clone().id);
        let model = models[0].clone();
        let queries: Vec<query::Model> = create_query(1, model.id);
        let query = queries[0].clone();
        let new_query = query::Model {
            outdated: false,
            ..query
        };

        user::Entity::insert(users[0].clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        model::Entity::insert(model.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        query::Entity::insert(new_query.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let new_model = model::Model {
            name: models[0].clone().name + "123",
            ..model
        };

        let updated_model = model_context.update(new_model.clone()).await.unwrap();

        let fetched_query = query::Entity::find_by_id(updated_model.id)
            .one(&db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(fetched_query.outdated, true);
    }

    #[tokio::test]
    async fn update_non_existing_id_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::Model]).await;
        let model_context = ModelContext::new(db_context.clone());

        let models: Vec<model::Model> = create_models(2, 1);
        let model = models[0].clone();

        let updated_model = model_context.update(model.clone()).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            updated_model.unwrap_err(),
            DbErr::RecordNotUpdated
        ));
    }

    #[tokio::test]
    async fn delete_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::Model, AnyEntity::User]).await;
        let model_context = ModelContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let models: Vec<model::Model> = create_models(2, users[0].clone().id);
        let model = models[0].clone();

        user::Entity::insert(users[0].clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        model::Entity::insert(model.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let deleted_model = model_context.delete(model.id).await.unwrap();

        let all_models = model::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(model, deleted_model);
        assert_eq!(all_models.len(), 0);
    }

    #[tokio::test]
    async fn delete_cascade_query_test() -> () {
        // Setting up database and user context
        let db_context =
            setup_db_with_entities(vec![AnyEntity::Model, AnyEntity::Query, AnyEntity::User]).await;
        let model_context = ModelContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let models: Vec<model::Model> = create_models(1, users[0].clone().id);
        let queries: Vec<query::Model> = create_query(1, models[0].clone().id);
        let model = models[0].clone();
        let query = queries[0].clone();

        user::Entity::insert(users[0].clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        model::Entity::insert(model.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        query::Entity::insert(query.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        model_context.delete(model.id).await.unwrap();

        let all_queries = query::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();
        let all_models = model::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_queries.len(), 0);
        assert_eq!(all_models.len(), 0);
    }

    #[tokio::test]
    async fn delete_cascade_access_test() -> () {
        // Setting up database and user context
        let db_context =
            setup_db_with_entities(vec![AnyEntity::Model, AnyEntity::Access, AnyEntity::User])
                .await;
        let model_context = ModelContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let models: Vec<model::Model> = create_models(1, users[0].clone().id);
        let accesses: Vec<access::Model> = create_accesses(1, 1, models[0].clone().id);

        let model = models[0].clone();
        let access = accesses[0].clone();

        user::Entity::insert(users[0].clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        model::Entity::insert(model.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        access::Entity::insert(access.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        model_context.delete(model.id).await.unwrap();

        let all_models = model::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();
        let all_accesses = access::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_models.len(), 0);
        assert_eq!(all_accesses.len(), 0);
    }

    #[tokio::test]
    async fn delete_cascade_in_use_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![
            AnyEntity::Model,
            AnyEntity::InUse,
            AnyEntity::User,
            AnyEntity::Session,
        ])
        .await;
        let model_context = ModelContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let sessions: Vec<session::Model> = create_sessions(1, users[0].clone().id);
        let models: Vec<model::Model> = create_models(1, users[0].clone().id);
        let in_uses: Vec<in_use::Model> = create_in_use(1, models[0].clone().id, 1);

        let model = models[0].clone();
        let in_use = in_uses[0].clone();

        user::Entity::insert(users[0].clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        session::Entity::insert(sessions[0].clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        model::Entity::insert(model.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        model_context.delete(model.id).await.unwrap();

        let all_models = model::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();
        let all_in_uses = in_use::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_models.len(), 0);
        assert_eq!(all_in_uses.len(), 0);
    }

    #[tokio::test]
    async fn delete_non_existing_id_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::Model]).await;
        let model_context = ModelContext::new(db_context.clone());

        let deleted_model = model_context.delete(1).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            deleted_model.unwrap_err(),
            DbErr::RecordNotFound(_)
        ));
    }
}
