use crate::entities::prelude::User;
use crate::entities::user::Entity;
use crate::tests::database::helpers::{create_entities, create_users};

#[cfg(test)]
mod database_tests {
    use crate::tests::database::helpers::*;
    use futures::FutureExt;
    use sea_orm::{entity::prelude::*, Database, IntoActiveModel};

    use crate::database::database_context::DatabaseContextTrait;
    use crate::{
        database::{
            database_context::DatabaseContext, entity_context::EntityContextTrait,
            user_context::UserContext,
        },
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

        let mut users_vec: Vec<UserModel> = vec![
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
    // TODO den skal slettes senere
    #[tokio::test]
    async fn create_test_test() -> () {
        let context = setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await;
        let user_context = UserContext::new(context);
        let users = two_template_users();
        let res = user_context.create(users[1].to_owned()).await.unwrap();
        assert_eq!(res, user_context.get_by_id(1).await.unwrap().unwrap())
    }

    #[tokio::test]
    async fn test_help() -> () {
        //let res = create_users(3);
        let vector: Vec<UserModel> = create_entities(3, |x| UserModel {
            id: &x + 1,
            email: format!("mail{}@mail.dk", &x),
            username: format!("username{}", &x),
            password: format!("qwerty{}", &x),
        });
        assert!(&vector[0].email == "mail0@mail.dk" && &vector[2].email == "mail2@mail.dk")
    }
}
