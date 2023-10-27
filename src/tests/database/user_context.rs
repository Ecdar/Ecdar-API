#[cfg(test)]
mod database_tests {
    use crate::tests::database::helpers::*;
    use sea_orm::{entity::prelude::*, IntoActiveModel};
    use std::matches;

    use crate::database::database_context::DatabaseContextTrait;
    use crate::entities::sea_orm_active_enums::Role;
    use crate::{
        database::{
            entity_context::EntityContextTrait,
            user_context::{DbErr, UserContext},
        },
        entities::access::{Entity as AccessEntity, Model as AccessModel},
        entities::model::{Entity as ModelEntity, Model as ModelModel},
        entities::session::{Entity as SessionEntity, Model as SessionModel},
        entities::user::{
            ActiveModel as UserActiveModel, Entity as UserEntity, Model as UserModel,
        },
    };

    fn user_generator(x: i32) -> UserModel {
        UserModel {
            id: &x + 1,
            email: format!("mail{}@mail.dk", &x),
            username: format!("username{}", &x),
            password: format!("qwerty{}", &x),
        }
    }

    // Test the functionality of the 'create' function, which creates a user in the database
    #[tokio::test]
    async fn create_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context);

        // Creates a model of the user which will be created

        let users: Vec<UserModel> = create_entities(1, user_generator);
        let new_user = users[0].clone();

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

        let users: Vec<UserModel> = create_entities(2, user_generator);

        // Creates the user in the database using the 'create' function
        let created_user1 = user_context.create(users[0].clone()).await.unwrap();
        let created_user2 = user_context.create(users[1].clone()).await.unwrap();

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

        let users: Vec<UserModel> = create_entities(1, user_generator);
        let new_user = users[0].clone();

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

        let users_vec: Vec<UserModel> = create_entities(1, user_generator);

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

    #[tokio::test]
    async fn update_non_existing_id_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let new_user = UserModel {
            id: 1,
            email: "anders21@student.aau.dk".to_string(),
            username: "andemad".to_owned(),
            password: "rask".to_owned(),
        };

        let updated_user = user_context.update(new_user.clone()).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            updated_user.unwrap_err(),
            DbErr::RecordNotFound(_)
        ));
    }

    #[tokio::test]
    async fn delete_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<UserModel> = create_entities(1, |x| UserModel {
            id: &x + 1,
            email: format!("mail{}@mail.dk", &x),
            username: format!("username{}", &x),
            password: format!("qwerty{}", &x),
        });

        let user = users[0].clone();

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

    #[tokio::test]
    async fn delete_cascade_model_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<UserModel> = create_entities(1, user_generator);
        let user = users[0].clone();

        let model = ModelModel {
            id: 1,
            name: "test".to_string(),
            components_info: "{}".to_owned().parse().unwrap(),
            owner_id: 1,
        };

        UserEntity::insert(user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        ModelEntity::insert(model.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        user_context.delete(user.id).await.unwrap();

        let all_users = UserEntity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();
        let all_models = ModelEntity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_users.len(), 0);
        assert_eq!(all_models.len(), 0);
    }

    #[tokio::test]
    async fn delete_access_model_test() -> () {
        // Setting up database and user context
        let db_context =
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model, AnyEntity::Access])
                .await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<UserModel> = create_entities(1, user_generator);
        let user = users[0].clone();

        let model = ModelModel {
            id: 1,
            name: "test".to_string(),
            components_info: "{}".to_owned().parse().unwrap(),
            owner_id: 1,
        };

        let access = AccessModel {
            id: 1,
            role: Role::Commenter,
            model_id: 1,
            user_id: 1,
        };

        UserEntity::insert(user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        ModelEntity::insert(model.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        AccessEntity::insert(access.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        user_context.delete(user.id).await.unwrap();

        let all_users = UserEntity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();
        let all_models = ModelEntity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();
        let all_accesses = AccessEntity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_users.len(), 0);
        assert_eq!(all_models.len(), 0);
        assert_eq!(all_accesses.len(), 0);
    }

    #[tokio::test]
    async fn delete_cascade_session_test() -> () {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Session]).await;
        let user_context = UserContext::new(db_context.clone());

        let users: Vec<UserModel> = create_entities(1, user_generator);

        let sessions: Vec<SessionModel> = create_entities(1, |x| SessionModel {
            id: &x + 1,
            token: Default::default(),
            created_at: Default::default(),
            user_id: 1,
        });

        let user = users[0].clone();
        let session = sessions[0].clone();

        UserEntity::insert(user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        SessionEntity::insert(session.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        user_context.delete(user.id).await.unwrap();

        let all_users = UserEntity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();
        let all_sessions = SessionEntity::find()
            .all(&db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_users.len(), 0);
        assert_eq!(all_sessions.len(), 0);
    }

    #[tokio::test]
    async fn delete_non_existing_id_test() {
        // Setting up database and user context
        let db_context = setup_db_with_entities(vec![AnyEntity::User]).await;
        let user_context = UserContext::new(db_context.clone());

        let deleted_user = user_context.delete(1).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            deleted_user.unwrap_err(),
            DbErr::RecordNotFound(_)
        ));
    }
}
