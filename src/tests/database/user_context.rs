#[cfg(test)]
mod database_tests {
    use crate::tests::database::helpers::*;
    use futures::FutureExt;
    use sea_orm::DbErr::Exec;
    use sea_orm::RuntimeErr::SqlxError;
    use sea_orm::{entity::prelude::*, Database, IntoActiveModel};
    use std::matches;

    use crate::database::database_context::DatabaseContextTrait;
    use crate::{
        database::{entity_context::EntityContextTrait, user_context::UserContext},
        entities::user::{
            ActiveModel as UserActiveModel, Entity as UserEntity, Model as UserModel,
        },
    };

    fn two_template_users() -> Vec<UserModel> {
        vec![
            UserModel {
                id: 1,
                email: "anders@mail.dk".to_string(),
                username: "anders".to_string(),
                password: "123".to_string(),
            },
            UserModel {
                id: 2,
                email: "mike@mail.dk".to_string(),
                username: "mikemanden".to_string(),
                password: "qwerty".to_string(),
            },
        ]
    }
    // Test the functionality of the 'create' function, which creates a user in the database
    #[tokio::test]
    async fn create_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context);

        // Creates a model of the user which will be created
        let new_user = UserModel {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad".to_owned(),
            password: "rask".to_owned(),
        };

        // Creates the user in the database using the 'create' function
        let created_user = user_context.create(new_user.clone()).await.unwrap();

        let fetched_user = UserEntity::find_by_id(created_user.id)
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
        let new_user1 = UserModel {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad".to_owned(),
            password: "rask".to_owned(),
        };

        let new_user2 = UserModel {
            id: 2,
            email: "anders22@student.aau.dk".to_owned(),
            username: "andemad".to_owned(),
            password: "rask".to_owned(),
        };

        // Creates the user in the database using the 'create' function
        let created_user1 = user_context.create(new_user1.clone()).await.unwrap();
        let created_user2 = user_context.create(new_user2.clone()).await;

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
        let new_user1 = UserModel {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad1".to_owned(),
            password: "rask".to_owned(),
        };

        let new_user2 = UserModel {
            id: 2,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad2".to_owned(),
            password: "rask".to_owned(),
        };

        // Creates the user in the database using the 'create' function
        let created_user1 = user_context.create(new_user1.clone()).await.unwrap();
        let created_user2 = user_context.create(new_user2.clone()).await;

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

        // Creates a model of the user which will be created
        let new_user1 = UserModel {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad1".to_owned(),
            password: "rask".to_owned(),
        };

        let new_user2 = UserModel {
            id: 1,
            email: "anders22@student.aau.dk".to_owned(),
            username: "andemad2".to_owned(),
            password: "rask".to_owned(),
        };

        // Creates the user in the database using the 'create' function
        let created_user1 = user_context.create(new_user1.clone()).await.unwrap();
        let created_user2 = user_context.create(new_user2.clone()).await.unwrap();

        let fetched_user1 = UserEntity::find_by_id(created_user1.id)
            .one(&user_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        let fetched_user2 = UserEntity::find_by_id(created_user2.id)
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

        // Creates a model of the user which will be created
        let new_user = UserModel {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad".to_owned(),
            password: "rask".to_owned(),
        };

        // Creates the user in the database using the 'create' function
        UserEntity::insert(new_user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        // Fetches the user created using the 'get_by_id' function
        let fetched_user = user_context.get_by_id(new_user.id).await.unwrap().unwrap();

        // Assert if the new_user, created_user, and fetched_user are the same
        assert_eq!(new_user, fetched_user);
    }
    #[tokio::test]
    async fn get_all_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let users_vec: Vec<UserModel> = vec![
            UserModel {
                id: 1,
                email: "anders21@student.aau.dk".to_string(),
                username: "anders".to_string(),
                password: "123".to_string(),
            },
            UserModel {
                id: 2,
                email: "mike@mail.dk".to_string(),
                username: "mikeManden".to_string(),
                password: "qwerty".to_string(),
            },
        ];

        let active_users_vec = users_vec
            .clone()
            .into_iter()
            .map(|x| x.into_active_model())
            .collect::<Vec<UserActiveModel>>();

        UserEntity::insert_many(active_users_vec)
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let result = user_context.get_all().await.unwrap();

        assert_eq!(users_vec, result);
    }

    #[tokio::test]
    async fn update_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let user = UserModel {
            id: 1,
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: "123".to_string(),
        };
        UserEntity::insert(user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let new_user = UserModel {
            password: "qwerty".to_string(),
            ..user
        };

        let updated_user = user_context.update(new_user.clone()).await.unwrap();

        let fetched_user = UserEntity::find_by_id(updated_user.id)
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

        // Creates a model of the user which will be created
        let new_user1 = UserModel {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad1".to_owned(),
            password: "rask".to_owned(),
        };

        let new_user2 = UserModel {
            id: 2,
            email: "anders22@student.aau.dk".to_owned(),
            username: "andemad2".to_owned(),
            password: "rask".to_owned(),
        };

        UserEntity::insert(new_user1.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        UserEntity::insert(new_user2.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let new_user = UserModel {
            username: "andemad2".to_string(),
            ..new_user1
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
        let new_user1 = UserModel {
            id: 1,
            email: "anders21@student.aau.dk".to_owned(),
            username: "andemad1".to_owned(),
            password: "rask".to_owned(),
        };

        let new_user2 = UserModel {
            id: 2,
            email: "anders22@student.aau.dk".to_owned(),
            username: "andemad2".to_owned(),
            password: "rask".to_owned(),
        };

        UserEntity::insert(new_user1.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        UserEntity::insert(new_user2.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let new_user = UserModel {
            email: "anders22@student.aau.dk".to_string(),
            ..new_user1
        };

        let updated_user = user_context.update(new_user.clone()).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            updated_user.unwrap_err().sql_err(),
            Some(SqlErr::UniqueConstraintViolation(_))
        ));
    }

    ///test that where the unique email constraint is violated
    #[tokio::test]
    async fn delete_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());
        let user = create_users(1)[0].clone();

        UserEntity::insert(user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let deleted_user = user_context.delete(user.id).await.unwrap();

        let all_users = UserEntity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(user, deleted_user);
        assert_eq!(all_users.len(), 0);
    }

    // Back up
    #[tokio::test]
    async fn update_fail() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());
        let mut users = two_template_users();

        for user in users.iter_mut() {
            let _ = user_context.create(user.to_owned()).await;
        }
        let res = user_context
            .update(UserModel {
                email: "mike@mail.dk".to_string(),
                ..users[0].to_owned()
            })
            .await;
        match res {
            Ok(_) => {
                panic!("should not happen")
            }
            Err(_err) => {
                return;
            }
        }
    }
    #[tokio::test]
    async fn delete_test_fail() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());
        let mut users = two_template_users();

        for user in users.iter_mut() {
            let _ = user_context.create(user.to_owned()).await;
        }
        let res = user_context.delete(3).await;
        match res {
            Ok(_) => {
                panic!("should not happen")
            }
            Err(_err) => {
                return;
            }
        }
    }
}
