use crate::database::user_context;
use crate::database::database_context;
use crate::database::entity_context;
use crate::entities::prelude::User;
use crate::entities::user::{ActiveModel, Model};

#[cfg(test)]
mod database_tests {
    use sea_orm::{
        entity::prelude::*, entity::*, tests_cfg::*,
        DatabaseBackend, MockDatabase, Transaction, DatabaseConnection, Database, Schema, sea_query::TableCreateStatement
    };
    use crate::{entities::user::{self, Model, Entity}, database::{database_context::{DatabaseContext, DatabaseContextTrait}, user_context::{self, UserContext}, entity_context::EntityContextTrait}};

    async fn setup_schema(db: &DatabaseConnection) {
        // Setup Schema helper
        let schema = Schema::new(DatabaseBackend::Sqlite);

        // Derive from Entity
        let stmt: TableCreateStatement = schema.create_table_from_entity(Entity);
        let _= db.execute(db.get_database_backend().build(&stmt)).await;

    }

    // Test the functionality of the 'create' function, which creates a user in the database
    #[tokio::test]
    async fn create_test() -> Result<(),DbErr> {

        // Setting up a sqlite database in memory to test on
        let db_connection = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&db_connection).await;
        let db_context = DatabaseContext { db: db_connection };
        let user_context = UserContext::new(db_context);

        // Creates a model of the user which will be created
        let new_user = Model {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad".to_owned(),
            password: "rask".to_owned(),
        };

        // Creates the user in the database using the 'create' function
        let created_user = user_context.create(new_user).await?;
        
        let fetched_user = Entity::find_by_id(created_user.id).one(&user_context.db_context.db).await?;

        // Assert if the fetched user is the same as the created user
        assert_eq!(fetched_user.unwrap().username, created_user.username);
        /*
        let db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([
            vec![user::Model{
                id: 1,
                email: "anders21@student.aau.dk".to_owned(),
                username: "andemad".to_owned(),
                password: "rask".to_owned(),
            }],
            vec![user::Model{
                id: 1,
                email: "anders21@student.aau.dk".to_owned(),
                username: "andemad".to_owned(),
                password: "rask".to_owned(),},
                user::Model{
                    id: 2,
                    email: "andeand@and.and".to_owned(),
                    username: "OgsåAndersRask".to_owned(),
                    password: "rask".to_owned(),
                }
            ]
        ]);
        */

        Ok(())
    }



}