#[cfg(test)]
mod database_tests {
    use crate::entities::prelude::User;
    use crate::{
        database::{
            database_context::DatabaseContext, entity_context::EntityContextTrait,
            user_context::UserContext,
        },
        entities::user::{self, Entity as UserEntity, Model},
    };
    use sea_orm::{
        entity::prelude::*, sea_query::TableCreateStatement, Database, DatabaseBackend,
        DatabaseConnection, Schema,
    };
    use std::any::Any;

    // TODO skal også flyttes så andre test filer kan gøre brug af den
    async fn setup_db_with_entities<E>(entities: Vec<E>) -> DatabaseContext
    where
        E: EntityTrait,
    {
        let connection = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(DatabaseBackend::Sqlite);
        for entity in entities.iter() {
            let stmt = schema.create_table_from_entity(entity.to_owned());
            let _ = connection
                .execute(connection.get_database_backend().build(&stmt))
                .await;
        }
        DatabaseContext { db: connection }
    }
    async fn setup_schema(db: &DatabaseConnection) {
        // Setup Schema helper
        let schema = Schema::new(DatabaseBackend::Sqlite);

        // Derive from Entity
        let stmt: TableCreateStatement = schema.create_table_from_entity(UserEntity);
        let _ = db.execute(db.get_database_backend().build(&stmt)).await;
    }

    /// Sets up a UserContext connected to an in-memory sqlite db
    async fn test_setup() -> UserContext {
        let connection = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&connection).await;
        let db_context = DatabaseContext { db: connection };
        UserContext::new(db_context)
    }
    fn two_template_users() -> Vec<Model> {
        vec![
            Model {
                id: 1,
                email: "anders@mail.dk".to_string(),
                username: "anders".to_string(),
                password: "123".to_string(),
            },
            Model {
                id: 2,
                email: "mike@mail.dk".to_string(),
                username: "mikemanden".to_string(),
                password: "qwerty".to_string(),
            },
        ]
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

        let fetched_user = UserEntity::find_by_id(created_user.id)
            .one(&user_context.db_context.db)
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
    #[tokio::test]
    async fn get_all_test() -> () {
        let user_context = test_setup().await;

        let mut users_vec: Vec<Model> = vec![
            Model {
                id: 1,
                email: "anders21@student.aau.dk".to_string(),
                username: "anders".to_string(),
                password: "123".to_string(),
            },
            Model {
                id: 2,
                email: "mike@mail.dk".to_string(),
                username: "mikeManden".to_string(),
                password: "qwerty".to_string(),
            },
        ];
        let mut res_users: Vec<Model> = vec![];
        for user in users_vec.iter_mut() {
            res_users.push(user_context.create(user.to_owned()).await.unwrap());
        }
        assert_eq!(users_vec, res_users);
    }

    #[tokio::test]
    async fn update_test() -> () {
        let user_context = test_setup().await;

        let user = Model {
            id: 1,
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: "123".to_string(),
        };
        let user = user_context.create(user).await.unwrap();
        let updated_user = Model {
            password: "qwerty".to_string(),
            ..user
        };
        assert_eq!(
            updated_user,
            user_context.update(updated_user.to_owned()).await.unwrap()
        )
    }

    ///test that where the unique email constraint is violated
    #[tokio::test]
    async fn update_fail() -> () {
        let user_context = test_setup().await;
        let mut users = two_template_users();

        for user in users.iter_mut() {
            let _ = user_context.create(user.to_owned()).await;
        }
        let res = user_context
            .update(Model {
                email: "mike@mail.dk".to_string(),
                ..users[0].to_owned()
            })
            .await;
        match res {
            Ok(_) => {
                panic!("should not happen")
            }
            Err(err) => {
                return;
            }
        }
    }
    #[tokio::test]
    async fn delete_test() -> () {
        let user_context = test_setup().await;
        let mut users = two_template_users();

        for user in users.iter_mut() {
            let _ = user_context.create(user.to_owned()).await;
        }
        assert_eq!(users[0], user_context.delete(users[0].id).await.unwrap())
    }
    #[tokio::test]
    async fn delete_test_fail() -> () {
        let user_context = test_setup().await;
        let mut users = two_template_users();

        for user in users.iter_mut() {
            let _ = user_context.create(user.to_owned()).await;
        }
        let res = user_context.delete(3).await;
        match res {
            Ok(_) => {
                panic!("should not happen")
            }
            Err(err) => {
                return;
            }
        }
    }
    // TODO den skal slettes senere
    #[tokio::test]
    async fn create_test_test() -> () {
        let context = setup_db_with_entities(vec![UserEntity]).await;
        let user_context = UserContext::new(context);
        let users = two_template_users();
        let res = user_context.create(users[1].to_owned()).await.unwrap();
        assert_eq!(res, user_context.get_by_id(1).await.unwrap().unwrap())
    }
}
