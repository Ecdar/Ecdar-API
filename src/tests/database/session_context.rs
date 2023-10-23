use crate::database::database_context;
use crate::database::entity_context;
use crate::database::session_context;
use crate::entities::prelude::Session;
use crate::entities::user::{ActiveModel, Model};

#[cfg(test)]
mod database_tests {
    use sea_orm::{
        entity::prelude::*, entity::*, sea_query::TableCreateStatement, tests_cfg::*, Database,
        DatabaseBackend, DatabaseConnection, MockDatabase, Schema, Transaction,
    };

    use crate::{
        database::{
            database_context::{DatabaseContext, DatabaseContextTrait},
            entity_context::EntityContextTrait,
            session_context::{self, SessionContext},
        },
        entities::session::{self, Entity, Model},
    };

    use chrono::{offset::Local, DateTime};

    async fn setup_schema(db: &DatabaseConnection) {
        let schema = Schema::new(DatabaseBackend::Sqlite);

        let stmt: TableCreateStatement = schema.create_table_from_entity(Entity);
        let _ = db.execute(db.get_database_backend().build(&stmt)).await;
    }

    async fn setup_session_context() -> SessionContext {
        let db_connection = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&db_connection).await;
        let db_context = DatabaseContext { db: db_connection };
        SessionContext::new(db_context)
    }

    #[tokio::test]
    async fn create_context() {
        let session_context = setup_session_context().await;

        let test = match session_context.db_context.db.ping().await {
            Ok(()) => true,
            Err(_) => false,
        };
        assert!(test)
    }

    #[tokio::test]
    async fn create_test() {
        // Setting up a sqlite database in memory.
        let session_context = setup_session_context().await;

        let new_session = Model {
            id: 1,
            token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: 1,
        };

        let created_session = session_context.create(new_session).await.unwrap();

        let fetched_session = Entity::find_by_id(created_session.id)
            .one(&session_context.db_context.db)
            .await
            .unwrap();

        assert_eq!(fetched_session.unwrap().token, created_session.token);
    }

    #[tokio::test]
    async fn get_by_id_test() {
        let session_context = setup_session_context().await;

        let new_session = Model {
            id: 1,
            token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: 1,
        };

        let created_session = session_context.create(new_session).await.unwrap();

        let fetched_session = session_context.get_by_id(1).await.unwrap().unwrap();

        assert_eq!(fetched_session.token, created_session.token);
    }

    #[tokio::test]
    async fn get_by_id_not_found_test() {
        let session_context = setup_session_context().await;

        let test = match session_context.get_by_id(1).await.unwrap() {
            None => true,
            Some(_) => false,
        };

        assert!(test);
    }

    #[tokio::test]
    async fn get_all_test() {
        let session_context = setup_session_context().await;

        // Create the sessions structs
        let session1 = Model {
            id: 1,
            token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: 1,
        };

        let session2 = Model {
            id: 2,
            token: Uuid::parse_str("75ecdf25-538c-4fe0-872d-525570c96b91").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: 1,
        };

        // Create the records in the database.
        session_context.create(session1).await.unwrap();
        session_context.create(session2).await.unwrap();

        let length = session_context.get_all().await.unwrap().len();

        assert_eq!(length, 2);
    }

    #[tokio::test]
    async fn get_all_not_found_test() {
        let session_context = setup_session_context().await;

        let test = session_context.get_all().await.unwrap().is_empty();

        assert!(test);
    }

    #[tokio::test]
    async fn update_test() {
        let session_context = setup_session_context().await;

        let original_session = Model {
            id: 1,
            token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: 1,
        };

        let original_session = session_context.create(original_session).await.unwrap();

        let altered_session = Model {
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
        let session_context = setup_session_context().await;

        let original_session = Model {
            id: 1,
            token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: 1,
        };

        let updated_context = session_context.update(original_session).await;

        assert!(updated_context.is_err());
    }

    #[tokio::test]
    async fn update_id_test() {
        let session_context = setup_session_context().await;

        let original_session = Model {
            id: 1,
            token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: 1,
        };

        let original_session = session_context.create(original_session).await.unwrap();

        let altered_session = Model {
            id: 2,
            ..original_session
        };

        let altered_session = session_context.update(altered_session).await.unwrap();

        let fetched_session = session_context.get_by_id(altered_session.id).await;

        assert!(fetched_session.is_err());
    }

    #[tokio::test]
    async fn delete_test() {
        let session_context = setup_session_context().await;

        let original_session = Model {
            id: 1,
            token: Uuid::parse_str("5c5e9172-9dff-4f35-afde-029a6f99652c").unwrap(),
            created_at: Local::now().naive_utc(),
            user_id: 1,
        };

        let session = session_context.create(original_session).await.unwrap();

        let deleted_session = session_context.delete(session.id).await.unwrap();

        assert_eq!(session.token, deleted_session.token);
    }

    #[tokio::test]
    async fn delete_not_found_test() {
        let session_context = setup_session_context().await;

        let result = session_context.delete(3).await;

        assert!(result.is_err());
    }
}

