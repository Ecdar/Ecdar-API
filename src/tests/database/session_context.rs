#[cfg(test)]
mod database_tests {
    use crate::tests::database::helpers::*;
    use sea_orm::{entity::prelude::*, IntoActiveModel};

    use crate::{
        database::{entity_context::EntityContextTrait, session_context::SessionContext},
        entities::{in_use, model, session, user},
    };

    use chrono::offset::Local;

    #[tokio::test]
    async fn create_context() {
        let session_context = SessionContext::new(
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session])
                .await
                .clone(),
        );

        let test = match session_context.db_context.get_connection().ping().await {
            Ok(()) => true,
            Err(_) => false,
        };
        assert!(test)
    }

    #[tokio::test]
    async fn create_test() {
        // Setting up a sqlite database in memory.
        let session_context = SessionContext::new(
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session])
                .await
                .clone(),
        );

        let users: Vec<user::Model> = create_entities(2, |x| user::Model {
            id: &x + 1,
            email: format!("mail{}@mail.dk", &x),
            username: format!("username{}", &x),
            password: format!("qwerty{}", &x),
        });

        user::Entity::insert_many(activate!(users, user::ActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let new_session = session::Model {
            id: 1,
            token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: users[0].id,
        };

        let created_session = session_context.create(new_session).await.unwrap();

        let fetched_session = session::Entity::find_by_id(created_session.id)
            .one(&session_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(fetched_session.unwrap().token, created_session.token);
    }

    #[tokio::test]
    async fn create_auto_increment_test() {
        // Setting up database and session context
        let db_context = setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session]).await;
        let session_context = SessionContext::new(db_context);

        let users: Vec<user::Model> = create_users(2);
        user::Entity::insert_many(activate!(users, user::ActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let sessions: Vec<session::Model> = create_entities(2, |x| session::Model {
            id: 1,
            token: Uuid::parse_str(format!("4473240f-2acb-422f-bd1a-5214554ed0e{}", &x).as_str())
                .unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: &x + 1,
        });

        // Creates the sessions in the database using the 'create' function
        let created_session1 = session_context.create(sessions[0].clone()).await.unwrap();
        let created_session2 = session_context.create(sessions[1].clone()).await.unwrap();

        let fetched_session1 = session::Entity::find_by_id(created_session1.id)
            .one(&session_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        let fetched_session2 = session::Entity::find_by_id(created_session2.id)
            .one(&session_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        // Assert if the new_session, created_session, and fetched_session are the same
        assert_ne!(fetched_session1.id, fetched_session2.id);
        assert_ne!(created_session1.id, created_session2.id);
        assert_eq!(created_session1.id, fetched_session1.id);
        assert_eq!(created_session2.id, fetched_session2.id);
    }

    #[tokio::test]
    async fn check_timestamp() {
        let session_context = SessionContext::new(
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session])
                .await
                .clone(),
        );

        let users: Vec<user::Model> = create_users(1);

        user::Entity::insert_many(activate!(users, user::ActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let new_session = session::Model {
            id: 1,
            token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: users[0].id,
        };

        let created_session = session_context.create(new_session.clone()).await.unwrap();

        let fetched_session = session::Entity::find_by_id(1)
            .one(&session_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(new_session.created_at, fetched_session.created_at);
        assert_eq!(new_session.created_at, created_session.created_at);
    }

    #[tokio::test]
    async fn get_by_id_test() {
        let session_context = SessionContext::new(
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session])
                .await
                .clone(),
        );

        let users: Vec<user::Model> = create_entities(2, |x| user::Model {
            id: &x + 1,
            email: format!("mail{}@mail.dk", &x),
            username: format!("username{}", &x),
            password: format!("qwerty{}", &x),
        });

        user::Entity::insert_many(activate!(users, user::ActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let new_session = session::Model {
            id: 1,
            token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: users[0].id,
        };

        let created_session = session_context.create(new_session).await.unwrap();

        let fetched_session = session::Entity::find_by_id(1)
            .one(&session_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(fetched_session.token, created_session.token);
    }

    #[tokio::test]
    async fn get_by_id_not_found_test() {
        let session_context = SessionContext::new(
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session])
                .await
                .clone(),
        );

        let test = match session_context.get_by_id(1).await.unwrap() {
            None => true,
            Some(_) => false,
        };

        assert!(test);
    }

    #[tokio::test]
    async fn get_all_test() {
        let session_context = SessionContext::new(
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session])
                .await
                .clone(),
        );

        let users: Vec<user::Model> = create_entities(2, |x| user::Model {
            id: &x + 1,
            email: format!("mail{}@mail.dk", &x),
            username: format!("username{}", &x),
            password: format!("qwerty{}", &x),
        });

        user::Entity::insert_many(activate!(users, user::ActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        // Create the sessions structs
        let sessions = create_entities(2, |x| session::Model {
            id: &x + 1,
            token: Uuid::parse_str(format!("5c5e9172-9dff-4f35-afde-029a6f99652{}", &x).as_str())
                .unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: 1,
        });

        session::Entity::insert_many(activate!(sessions, session::ActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let length = session_context.get_all().await.unwrap().len();

        assert_eq!(length, 2);
    }

    #[tokio::test]
    async fn get_all_not_found_test() {
        let session_context = SessionContext::new(
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session])
                .await
                .clone(),
        );

        let test = session_context.get_all().await.unwrap().is_empty();

        assert!(test);
    }

    #[tokio::test]
    async fn update_test() {
        let session_context = SessionContext::new(
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session])
                .await
                .clone(),
        );

        let users = create_users(2);

        user::Entity::insert_many(activate!(users, user::ActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let original_session = session::Model {
            id: 1,
            token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: users[0].id,
        };

        session::Entity::insert(original_session.clone().into_active_model())
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let altered_session = session::Model {
            token: Uuid::parse_str("ddd9b7a3-98ff-43b0-b5b5-aa2abaea9d96").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: 2,
            ..original_session
        };

        let altered_session = session_context.update(altered_session).await.unwrap();

        let fetched_session = session_context
            .get_by_id(altered_session.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(original_session, fetched_session);
    }

    #[tokio::test]
    async fn delete_test() {
        let session_context = SessionContext::new(
            setup_db_with_entities(vec![
                AnyEntity::User,
                AnyEntity::Session,
                AnyEntity::InUse,
                AnyEntity::Model,
            ])
            .await
            .clone(),
        );

        let users = create_users(2);

        user::Entity::insert_many(activate!(users, user::ActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let original_session = session::Model {
            id: 1,
            token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: 1,
        };

        session::Entity::insert(original_session.clone().into_active_model())
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let model = create_models(1, 1);

        model::Entity::insert(model[0].clone().into_active_model())
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let inuse = create_in_use(1, 1, 1);

        in_use::Entity::insert(inuse[0].clone().into_active_model())
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let deleted_session = session_context.delete(original_session.id).await.unwrap();

        let in_use_length = in_use::Entity::find()
            .all(&session_context.db_context.get_connection())
            .await
            .unwrap()
            .len();

        assert_eq!(original_session, deleted_session);
        assert_eq!(in_use_length, 0);
    }

    #[tokio::test]
    async fn delete_not_found_test() {
        let session_context = SessionContext::new(
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session])
                .await
                .clone(),
        );

        let result = session_context.delete(3).await;

        assert!(result.is_err());
    }
}
