#[cfg(test)]
mod database_tests {
    use crate::tests::database::helpers::*;
    use sea_orm::{
        entity::prelude::*, sea_query::TableCreateStatement, ActiveValue::Set, Database,
        DatabaseBackend, DatabaseConnection, IntoActiveModel, Schema,
    };

    use crate::{
        database::{
            database_context::DatabaseContext, entity_context::EntityContextTrait,
            session_context::SessionContext,
        },
        entities::session::{
            ActiveModel as SessionActiveModel, Entity as SessionEntity, Model as SessionModel,
        },
        entities::user::{
            ActiveModel as UserActiveModel, Entity as UserEntity, Model as UserModel,
        },
    };

    use chrono::offset::Local;

    async fn setup_schema(db: &DatabaseConnection) {
        let schema = Schema::new(DatabaseBackend::Sqlite);

        let session_stmt: TableCreateStatement = schema.create_table_from_entity(SessionEntity);
        let user_stmt: TableCreateStatement =
            schema.create_table_from_entity(crate::entities::user::Entity);
        db.execute(db.get_database_backend().build(&session_stmt))
            .await
            .unwrap();

        db.execute(db.get_database_backend().build(&user_stmt))
            .await
            .unwrap();
    }

    async fn setup_session_context() -> (SessionContext, i32) {
        let db_connection = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&db_connection).await;

        // Create standard user and return the id.
        let user = crate::entities::user::ActiveModel {
            id: Default::default(),
            email: Set("DUNK".into()),
            username: Set("DUNK".into()),
            password: Set("DUNK".into()),
        };

        let user = user.insert(&db_connection).await.unwrap();

        let db_context = DatabaseContext { db_connection };

        (SessionContext::new(Box::new(db_context)), user.id)
    }

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

        let users: Vec<UserModel> = create_entities(2, |x| UserModel {
            id: &x + 1,
            email: format!("mail{}@mail.dk", &x),
            username: format!("username{}", &x),
            password: format!("qwerty{}", &x),
        });

        UserEntity::insert_many(activate!(users, UserActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let new_session = SessionModel {
            id: 1,
            token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: users[0].id,
        };

        let created_session = session_context.create(new_session).await.unwrap();

        let fetched_session = SessionEntity::find_by_id(1)
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

        let users: Vec<UserModel> = create_users(2);
        UserEntity::insert_many(activate!(users, UserActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let sessions: Vec<SessionModel> = create_entities(2, |x| SessionModel {
            id: 1,
            token: Uuid::parse_str(format!("4473240f-2acb-422f-bd1a-5214554ed0e{}", &x).as_str())
                .unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: &x + 1,
        });

        // Creates the sessions in the database using the 'create' function
        let created_session1 = session_context.create(sessions[0].clone()).await.unwrap();
        let created_session2 = session_context.create(sessions[1].clone()).await.unwrap();

        let fetched_session1 = SessionEntity::find_by_id(created_session1.id)
            .one(&session_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        let fetched_session2 = SessionEntity::find_by_id(created_session2.id)
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

        let users: Vec<UserModel> = create_users(1);

        UserEntity::insert_many(activate!(users, UserActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let new_session = SessionModel {
            id: 1,
            token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: users[0].id,
        };

        let created_session = session_context.create(new_session.clone()).await.unwrap();

        let fetched_session = SessionEntity::find_by_id(1)
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

        let users: Vec<UserModel> = create_entities(2, |x| UserModel {
            id: &x + 1,
            email: format!("mail{}@mail.dk", &x),
            username: format!("username{}", &x),
            password: format!("qwerty{}", &x),
        });

        UserEntity::insert_many(activate!(users, UserActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let new_session = SessionModel {
            id: 1,
            token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: users[0].id,
        };

        let created_session = session_context.create(new_session).await.unwrap();

        let fetched_session = SessionEntity::find_by_id(1)
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

        let users: Vec<UserModel> = create_entities(2, |x| UserModel {
            id: &x + 1,
            email: format!("mail{}@mail.dk", &x),
            username: format!("username{}", &x),
            password: format!("qwerty{}", &x),
        });

        UserEntity::insert_many(activate!(users, UserActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let sessions: Vec<SessionModel> = create_entities(2, |x| SessionModel {
            id: &x + 1,
            token: Uuid::parse_str(format!("4473240f-2acb-422f-bd1a-5214554ed0e{}", &x).as_str())
                .unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: &x + 1,
        });

        SessionEntity::insert_many(activate!(sessions, SessionActiveModel))
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

        UserEntity::insert_many(activate!(users, UserActiveModel))
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let original_session = SessionModel {
            id: 1,
            token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: users[0].id,
        };

        let original_session = session_context.create(original_session).await.unwrap();

        let altered_session = SessionModel {
            token: Uuid::parse_str("ddd9b7a3-98ff-43b0-b5b5-aa2abaea9d96").unwrap(),
            ..original_session
        };

        let altered_session = session_context.update(altered_session).await.unwrap();

        let fetched_session = session_context
            .get_by_id(altered_session.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(altered_session.token, fetched_session.token);
    }

    #[tokio::test]
    async fn update_test_failed_test() {
        let session_context = SessionContext::new(
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session])
                .await
                .clone(),
        );

        let user = &create_users(1)[0];

        UserEntity::insert(user.clone().into_active_model())
            .exec(&session_context.db_context.get_connection())
            .await
            .unwrap();

        let original_session = SessionModel {
            id: 1,
            token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: user.id,
        };

        let updated_context = session_context.update(original_session).await;

        assert!(updated_context.is_err());
    }

    #[tokio::test]
    async fn update_id_test() {
        let (session_context, user_id) = setup_session_context().await;

        let original_session = SessionModel {
            id: 1,
            token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id,
        };

        let original_session = session_context.create(original_session).await.unwrap();

        let altered_session = SessionModel {
            id: 2,
            ..original_session
        };

        let altered_session = session_context.update(altered_session).await;

        assert!(altered_session.is_err());
    }

    #[tokio::test]
    async fn delete_test() {
        let (session_context, user_id) = setup_session_context().await;

        let original_session = SessionModel {
            id: 1,
            token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id,
        };

        let session = session_context.create(original_session).await.unwrap();

        let deleted_session = session_context.delete(session.id).await.unwrap();

        assert_eq!(session.token, deleted_session.token);
    }

    #[tokio::test]
    async fn delete_not_found_test() {
        let (session_context, _user_id) = setup_session_context().await;

        let result = session_context.delete(3).await;

        assert!(result.is_err());
    }
}

