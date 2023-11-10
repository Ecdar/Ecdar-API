#[cfg(test)]
mod database_tests {
    use sea_orm::{
        entity::prelude::*, sea_query::TableCreateStatement, ActiveValue::Set, Database,
        DatabaseBackend, DatabaseConnection, Schema,
    };

    use crate::{
        database::{
            database_context::DatabaseContext, entity_context::EntityContextTrait,
            session_context::SessionContext,
        },
        entities::session::{Entity, Model},
    };

    use chrono::offset::Local;

    async fn setup_schema(db: &DatabaseConnection) {
        let schema = Schema::new(DatabaseBackend::Sqlite);

        let session_stmt: TableCreateStatement = schema.create_table_from_entity(Entity);
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
        let (session_context, _user_id) = setup_session_context().await;

        let test = match session_context.db_context.get_connection().ping().await {
            Ok(()) => true,
            Err(_) => false,
        };
        assert!(test)
    }

    #[tokio::test]
    async fn create_test() {
        // Setting up a sqlite database in memory.
        let (session_context, user_id) = setup_session_context().await;

        let new_session = Model {
            id: 1,
            refresh_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            access_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            updated_at: Local::now().naive_utc(),
            user_id,
        };

        let created_session = session_context.create(new_session).await.unwrap();

        let fetched_session = Entity::find_by_id(created_session.id)
            .one(&session_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(
            fetched_session.unwrap().access_token,
            created_session.access_token
        );
    }

    #[tokio::test]
    async fn get_by_id_test() {
        let (session_context, user_id) = setup_session_context().await;

        let new_session = Model {
            id: 1,
            refresh_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            access_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            updated_at: Local::now().naive_utc(),
            user_id,
        };

        let created_session = session_context.create(new_session).await.unwrap();

        let fetched_session = session_context.get_by_id(1).await.unwrap().unwrap();

        assert_eq!(fetched_session.access_token, created_session.access_token);
    }

    #[tokio::test]
    async fn get_by_id_not_found_test() {
        let (session_context, _user_id) = setup_session_context().await;

        let test = session_context.get_by_id(1).await.unwrap().is_none();

        assert!(test);
    }

    #[tokio::test]
    async fn get_all_test() {
        let (session_context, user_id) = setup_session_context().await;

        // Create the sessions structs
        let session1 = Model {
            id: 1,
            refresh_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            access_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            updated_at: Local::now().naive_utc(),
            user_id,
        };

        let session2 = Model {
            id: 2,
            refresh_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e1").unwrap(),
            access_token: Uuid::parse_str("75ecdf25-538c-4fe0-872d-525570c96b91").unwrap(),
            updated_at: Local::now().naive_utc(),
            user_id,
        };

        // Create the records in the database.
        session_context.create(session1).await.unwrap();
        session_context.create(session2).await.unwrap();

        let length = session_context.get_all().await.unwrap().len();

        assert_eq!(length, 2);
    }

    #[tokio::test]
    async fn get_all_not_found_test() {
        let (session_context, _user_id) = setup_session_context().await;

        let test = session_context.get_all().await.unwrap().is_empty();

        assert!(test);
    }

    #[tokio::test]
    async fn update_test() {
        let (session_context, user_id) = setup_session_context().await;

        let original_session = Model {
            id: 1,
            refresh_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            access_token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            updated_at: Local::now().naive_utc(),
            user_id,
        };

        let original_session = session_context.create(original_session).await.unwrap();

        let altered_session = Model {
            refresh_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e1").unwrap(),
            access_token: Uuid::parse_str("ddd9b7a3-98ff-43b0-b5b5-aa2abaea9d96").unwrap(),
            ..original_session
        };

        let altered_session = session_context.update(altered_session).await.unwrap();

        let fetched_session = session_context
            .get_by_id(altered_session.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(altered_session.access_token, fetched_session.access_token);
    }

    #[tokio::test]
    async fn update_test_failed_test() {
        let (session_context, user_id) = setup_session_context().await;

        let original_session = Model {
            id: 1,
            refresh_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e1").unwrap(),
            access_token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            updated_at: Local::now().naive_utc(),
            user_id,
        };

        let updated_context = session_context.update(original_session).await;

        assert!(updated_context.is_err());
    }

    #[tokio::test]
    async fn update_id_test() {
        let (session_context, user_id) = setup_session_context().await;

        let original_session = Model {
            id: 1,
            refresh_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e1").unwrap(),
            access_token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            updated_at: Local::now().naive_utc(),
            user_id,
        };

        let original_session = session_context.create(original_session).await.unwrap();

        let altered_session = Model {
            id: 2,
            ..original_session
        };

        let altered_session = session_context.update(altered_session).await;

        assert!(altered_session.is_err());
    }

    #[tokio::test]
    async fn delete_test() {
        let (session_context, user_id) = setup_session_context().await;

        let original_session = Model {
            id: 1,
            refresh_token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e1").unwrap(),
            access_token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            updated_at: Local::now().naive_utc(),
            user_id,
        };

        let session = session_context.create(original_session).await.unwrap();

        let deleted_session = session_context.delete(session.id).await.unwrap();

        assert_eq!(session.access_token, deleted_session.access_token);
    }

    #[tokio::test]
    async fn delete_not_found_test() {
        let (session_context, _user_id) = setup_session_context().await;

        let result = session_context.delete(3).await;

        assert!(result.is_err());
    }
}
