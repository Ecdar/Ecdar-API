#[cfg(test)]
mod database_tests {
    use crate::{
        database::{
            database_context::DatabaseContext, entity_context::EntityContextTrait,
            user_context::UserContext,
        },
        entities::user::{Entity, Model as User},
    };
    use sea_orm::{
        entity::prelude::*, sea_query::TableCreateStatement, Database, DatabaseBackend,
        DatabaseConnection, Schema,
    };

    async fn setup_schema(db: &DatabaseConnection) {
        // Setup Schema helper
        let schema = Schema::new(DatabaseBackend::Sqlite);

        // Derive from Entity
        let stmt: TableCreateStatement = schema.create_table_from_entity(Entity);
        let _ = db.execute(db.get_database_backend().build(&stmt)).await;
    }

    // Test the functionality of the 'create' function, which creates a user in the database
    #[tokio::test]
    async fn create_test() -> Result<(), DbErr> {
        // Setting up a sqlite database in memory to test on
        let db_connection = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&db_connection).await;
        let db_context = Box::new(DatabaseContext { db_connection });
        let user_context = UserContext::new(db_context);

        // Creates a model of the user which will be created
        let new_user = User {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad".to_owned(),
            password: "rask".to_owned(),
        };

        // Creates the user in the database using the 'create' function
        let created_user = user_context.create(new_user).await?;

        let fetched_user = Entity::find_by_id(created_user.id)
            .one(&user_context.db_context.get_connection())
            .await?;

        // Assert if the fetched user is the same as the created user
        assert_eq!(fetched_user.unwrap().username, created_user.username);

        Ok(())
    }

    #[tokio::test]
    async fn get_by_id_test() -> Result<(), DbErr> {
        // Setting up a sqlite database in memory to test on
        let db_connection = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&db_connection).await;
        let db_context = Box::new(DatabaseContext {
            db_connection: db_connection,
        });
        let user_context = UserContext::new(db_context);

        // Creates a model of the user which will be created
        let new_user = User {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad".to_owned(),
            password: "rask".to_owned(),
        };

        // Creates the user in the database using the 'create' function
        let created_user = user_context.create(new_user).await?;

        // Fetches the user created using the 'get_by_id' function
        let fetched_user = user_context.get_by_id(created_user.id).await?;

        // Assert if the fetched user is the same as the created user
        assert_eq!(fetched_user.unwrap().username, created_user.username);

        Ok(())
    }
}
