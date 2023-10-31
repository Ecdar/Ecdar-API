#[cfg(test)]
mod database_tests {
    use crate::tests::database::helpers::*;
    use sea_orm::{entity::prelude::*, IntoActiveModel};
    use std::matches;

    use crate::database::database_context::DatabaseContextTrait;
    use crate::{
        database::{
            entity_context::EntityContextTrait,
            user_context::{DbErr, UserContext},
        },
        entities::{access, model, session, user},
        to_active_models,
    };

    // Test the functionality of the 'create' function, which creates a user in the database
    #[tokio::test]
    async fn create_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context);

        // Creates a model of the user which will be created
        let users: Vec<user::Model> = create_users(1);
        let new_user = users[0].clone();

        // Creates the user in the database using the 'create' function
        let created_user = user_context.create(new_user.clone()).await.unwrap();

        let fetched_user = user::Entity::find_by_id(created_user.id)
            .one(&user_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        // Assert if the new_user, created_user, and fetched_user are the same
        assert_eq!(new_user, created_user);
        assert_eq!(created_user, fetched_user);
    }

    #[tokio::test]
    async fn create_non_unique_username_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context);

        // Creates a model of the user which will be created
        let users: Vec<user::Model> = create_users(2);
        users[0].clone().username = users[1].clone().username;

        // Creates the user in the database using the 'create' function
        let _created_user1 = user_context.create(users[0].clone()).await.unwrap();
        let created_user2 = user_context.create(users[0].clone()).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            created_user2.unwrap_err().sql_err(),
            Some(SqlErr::UniqueConstraintViolation(_))
        ));
    }

    #[tokio::test]
    async fn create_non_unique_email_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context);

        // Creates a model of the user which will be created
        let users: Vec<user::Model> = create_users(2);
        users[0].clone().email = users[1].clone().email;

        // Creates the user in the database using the 'create' function
        let _created_user1 = user_context.create(users[0].clone()).await.unwrap();
        let created_user2 = user_context.create(users[0].clone()).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            created_user2.unwrap_err().sql_err(),
            Some(SqlErr::UniqueConstraintViolation(_))
        ));
    }

    #[tokio::test]
    async fn create_auto_increment_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context);

        let users: Vec<user::Model> = create_users(2);

        // Creates the user in the database using the 'create' function
        let created_user1 = user_context.create(users[0].clone()).await.unwrap();
        let created_user2 = user_context.create(users[1].clone()).await.unwrap();

        let fetched_user1 = user::Entity::find_by_id(created_user1.id)
            .one(&user_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        let fetched_user2 = user::Entity::find_by_id(created_user2.id)
            .one(&user_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        // Assert if the new_user, created_user, and fetched_user are the same
        assert_ne!(fetched_user1.id, fetched_user2.id);
        assert_ne!(created_user1.id, created_user2.id);
        assert_eq!(created_user1.id, fetched_user1.id);
        assert_eq!(created_user2.id, fetched_user2.id);
    }

    #[tokio::test]
    async fn get_by_id_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let new_user = users[0].clone();

        // Creates the user in the database using the 'create' function
        user::Entity::insert(new_user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        // Fetches the user created using the 'get_by_id' function
        let fetched_user = user_context.get_by_id(new_user.id).await.unwrap().unwrap();

        // Assert if the new_user, created_user, and fetched_user are the same
        assert_eq!(new_user, fetched_user);
    }

    #[tokio::test]
    async fn get_by_non_existing_id_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        // Fetches the user created using the 'get_by_id' function
        let fetched_user = user_context.get_by_id(1).await.unwrap();

        assert!(fetched_user.is_none());
    }

    #[tokio::test]
    async fn get_all_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let users_vec: Vec<user::Model> = create_users(1);

        let active_users_vec = to_active_models!(users_vec.clone());

        user::Entity::insert_many(active_users_vec)
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let result = user_context.get_all().await.unwrap();

        assert_eq!(users_vec, result);
    }

    #[tokio::test]
    async fn get_all_empty_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let result = user_context.get_all().await.unwrap();
        let empty_users: Vec<user::Model> = vec![];

        assert_eq!(empty_users, result);
    }

    #[tokio::test]
    async fn update_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(2);
        let user = users[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let new_user = user::Model {
            password: users[0].clone().password + "123",
            ..user
        };

        let updated_user = user_context.update(new_user.clone()).await.unwrap();

        let fetched_user = user::Entity::find_by_id(updated_user.id)
            .one(&db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(new_user, updated_user);
        assert_eq!(updated_user, fetched_user);
    }

    #[tokio::test]
    async fn update_non_unique_username_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(2);

        user::Entity::insert_many(to_active_models!(users.clone()))
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let new_user = user::Model {
            username: users[1].clone().username,
            ..users[0].clone()
        };

        let updated_user = user_context.update(new_user.clone()).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            updated_user.unwrap_err().sql_err(),
            Some(SqlErr::UniqueConstraintViolation(_))
        ));
    }

    #[tokio::test]
    async fn update_non_unique_email_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        // Creates a model of the user which will be created
        let users: Vec<user::Model> = create_users(2);

        user::Entity::insert_many(to_active_models!(users.clone()))
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let new_user = user::Model {
            email: users[1].clone().email,
            ..users[0].clone()
        };

        let updated_user = user_context.update(new_user.clone()).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            updated_user.unwrap_err().sql_err(),
            Some(SqlErr::UniqueConstraintViolation(_))
        ));
    }

    #[tokio::test]
    async fn update_non_existing_id_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(2);
        let user = users[0].clone();

        let updated_user = user_context.update(user.clone()).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(updated_user.unwrap_err(), DbErr::RecordNotUpdated));
    }

    #[tokio::test]
    async fn delete_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let user = users[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let deleted_user = user_context.delete(user.id).await.unwrap();

        let all_users = user::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(user, deleted_user);
        assert_eq!(all_users.len(), 0);
    }

    #[tokio::test]
    async fn delete_cascade_model_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let models: Vec<model::Model> = create_models(1, users[0].clone().id);

        let user = users[0].clone();
        let model = models[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        model::Entity::insert(model.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        user_context.delete(user.id).await.unwrap();

        let all_users = user::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();
        let all_models = model::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_users.len(), 0);
        assert_eq!(all_models.len(), 0);
    }

    #[tokio::test]
    async fn delete_access_model_test() -> () {
        // Setting up database and user context
        let db_context =
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model, AnyEntity::Access])
                .await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let models: Vec<model::Model> = create_models(1, users[0].clone().id);
        let accesses: Vec<access::Model> =
            create_accesses(1, users[0].clone().id, models[0].clone().id);

        let user = users[0].clone();
        let model = models[0].clone();
        let access = accesses[0].clone();

        user::Entity::insert(user.clone().into_active_model())
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

        user_context.delete(user.id).await.unwrap();

        let all_users = user::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();
        let all_models = model::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();
        let all_accesses = access::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_users.len(), 0);
        assert_eq!(all_models.len(), 0);
        assert_eq!(all_accesses.len(), 0);
    }

    #[tokio::test]
    async fn delete_cascade_session_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session]).await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<user::Model> = create_users(1);
        let sessions: Vec<session::Model> = create_sessions(1, users[0].clone().id);

        let user = users[0].clone();
        let session = sessions[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        session::Entity::insert(session.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        user_context.delete(user.id).await.unwrap();

        let all_users = user::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();
        let all_sessions = session::Entity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_users.len(), 0);
        assert_eq!(all_sessions.len(), 0);
    }

    #[tokio::test]
    async fn delete_non_existing_id_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let deleted_user = user_context.delete(1).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            deleted_user.unwrap_err(),
            DbErr::RecordNotFound(_)
        ));
    }
}
