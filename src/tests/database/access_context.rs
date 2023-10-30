#[cfg(test)]
mod database_tests {
    use crate::{
        database::{
            access_context::AccessContext, database_context::DatabaseContext,
            entity_context::EntityContextTrait, model_context::ModelContext,
            user_context::UserContext,
        },
        entities::{access, model, sea_orm_active_enums::Role, user},
    };
    use sea_orm::{
        entity::prelude::*, sea_query::TableCreateStatement, Database, DatabaseBackend,
        DatabaseConnection, Schema,
    };

    async fn setup_schema(db: &DatabaseConnection) {
        // Setup Schema helper
        let schema = Schema::new(DatabaseBackend::Sqlite);

        // Derive from Entity
        let stmt: TableCreateStatement = schema.create_table_from_entity(user::Entity);
        let _ = db.execute(db.get_database_backend().build(&stmt)).await;
        let stmt: TableCreateStatement = schema.create_table_from_entity(model::Entity);
        let _ = db.execute(db.get_database_backend().build(&stmt)).await;
        let stmt: TableCreateStatement = schema.create_table_from_entity(access::Entity);
        let _ = db.execute(db.get_database_backend().build(&stmt)).await;
    }

    // Test the functionality of the 'create' function, which creates a access in the database
    #[tokio::test]
    async fn create_test() -> Result<(), DbErr> {
        // Setting up a sqlite database in memory to test on
        let db_connection = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&db_connection).await;
        let db_context = Box::new(DatabaseContext { db_connection });

        let user_context = UserContext::new(db_context.clone());
        let model_context = ModelContext::new(db_context.clone());
        let access_context = AccessContext::new(db_context.clone());

        let new_user = user::Model {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad".to_owned(),
            password: "rask".to_owned(),
        };

        let new_model = model::Model {
            id: 1,
            name: "System".to_owned(),
            components_info: "{}".to_owned().parse().unwrap(),
            owner_id: 1,
        };

        // Creates a model of the access which will be created
        let new_access = access::Model {
            id: 1,
            role: Role::Editor,
            user_id: 1,
            model_id: 1,
        };

        // Creates the access in the database using the 'create' function
        user_context.create(new_user).await?;
        model_context.create(new_model).await?;
        let created_access = access_context.create(new_access).await?;

        let fetched_access = access::Entity::find_by_id(created_access.id)
            .one(&access_context.db_context.get_connection())
            .await?
            .clone()
            .unwrap();

        // Assert if the fetched access is the same as the created access
        assert_eq!(fetched_access.id, created_access.id);
        assert_eq!(fetched_access.user_id, created_access.user_id);
        assert_eq!(fetched_access.model_id, created_access.model_id);

        Ok(())
    }

    #[tokio::test]
    async fn get_by_id_test() -> Result<(), DbErr> {
        // Setting up a sqlite database in memory to test on
        let db_connection = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&db_connection).await;
        let db_context = Box::new(DatabaseContext { db_connection });
        let user_context = UserContext::new(db_context.clone());
        let model_context = ModelContext::new(db_context.clone());
        let access_context = AccessContext::new(db_context.clone());

        let new_user = user::Model {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad".to_owned(),
            password: "rask".to_owned(),
        };

        let new_model = model::Model {
            id: 1,
            name: "System".to_owned(),
            components_info: "{}".to_owned().parse().unwrap(),
            owner_id: 1,
        };

        // Creates a model of the access which will be created
        let new_access = access::Model {
            id: 1,
            role: Role::Editor,
            user_id: 1,
            model_id: 1,
        };

        // Creates the access in the database using the 'create' function
        user_context.create(new_user).await?;
        model_context.create(new_model).await?;
        let created_access = access_context.create(new_access).await?;

        // Fetches the access created using the 'get_by_id' function
        let fetched_access = access_context
            .get_by_id(created_access.id)
            .await?
            .clone()
            .unwrap();

        // Assert if the fetched access is the same as the created access
        assert_eq!(fetched_access.id, created_access.id);
        assert_eq!(fetched_access.user_id, created_access.user_id);
        assert_eq!(fetched_access.model_id, created_access.model_id);

        Ok(())
    }
}
