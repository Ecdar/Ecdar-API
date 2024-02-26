use crate::contexts::{DatabaseContextTrait, EntityContextTrait};
use crate::entities::user;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};
use std::sync::Arc;

#[async_trait]
pub trait UserContextTrait: EntityContextTrait<user::Model> {
    /// Searches for a `User` by username, returning [`Some`] if one is found, [`None`] otherwise.
    /// # Errors
    /// Errors on failed connection, execution error or constraint violations.
    /// # Notes
    /// Since usernames are unique, it is guaranteed that at most one user with the given username exists.
    async fn get_by_username(&self, username: String) -> Result<Option<user::Model>, DbErr>;
    /// Searches for a `User` by email address, returning [`Some`] if one is found, [`None`] otherwise.
    /// # Errors
    /// Errors on failed connection, execution error or constraint violations.
    /// # Notes
    /// Since email address' are unique, it is guaranteed that at most one user with the given email address exists.
    async fn get_by_email(&self, email: String) -> Result<Option<user::Model>, DbErr>;
    /// Returns all the user entities with the given ids
    /// # Example
    /// ```
    /// let context : UserContext = UserContext::new(...);
    /// let model : vec<Model> = context.get_by_ids(vec![1,2]).unwrap();
    /// assert_eq!(model.len(),2);
    /// ```
    async fn get_by_ids(&self, ids: Vec<i32>) -> Result<Vec<user::Model>, DbErr>;
}

pub struct UserContext {
    db_context: Arc<dyn DatabaseContextTrait>,
}

#[async_trait]
impl UserContextTrait for UserContext {
    async fn get_by_username(&self, username: String) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(&self.db_context.get_connection())
            .await
    }
    async fn get_by_email(&self, email: String) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db_context.get_connection())
            .await
    }

    async fn get_by_ids(&self, ids: Vec<i32>) -> Result<Vec<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::Id.is_in(ids))
            .all(&self.db_context.get_connection())
            .await
    }
}

impl UserContext {
    pub fn new(db_context: Arc<dyn DatabaseContextTrait>) -> UserContext {
        UserContext { db_context }
    }
}

#[async_trait]
impl EntityContextTrait<user::Model> for UserContext {
    /// Used for creating a User entity
    /// # Example
    /// ```
    /// let user : Model = {
    ///     id: Default::default(),
    ///     email: "anders@aau.dk".into(),
    ///     username: "Anders".into(),
    ///     password: "qwerty".into()
    /// }
    /// let context : UserContext = UserContext::new(...);
    /// context.create(user);
    /// ```
    async fn create(&self, entity: user::Model) -> Result<user::Model, DbErr> {
        user::ActiveModel {
            id: Default::default(),
            email: Set(entity.email),
            username: Set(entity.username),
            password: Set(entity.password),
        }
        .insert(&self.db_context.get_connection())
        .await
    }

    /// Returns a single user entity (uses primary key)
    /// # Example
    /// ```
    /// let context : UserContext = UserContext::new(...);
    /// let user : Model = context.get_by_id(1).unwrap();
    /// assert_eq!(user.username,"Anders".into());
    /// ```
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    /// Returns all the user entities
    /// # Example
    /// ```
    /// let context : UserContext = UserContext::new(...);
    /// let user : vec<Model> = context.get_all().unwrap();
    /// assert_eq!(user.len(),1);
    /// ```
    async fn get_all(&self) -> Result<Vec<user::Model>, DbErr> {
        user::Entity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    /// Updates and returns the given user entity
    /// # Example
    /// ```
    /// let context : UserContext = UserContext::new(...);
    /// let user = context.get_by_id(1).unwrap();
    /// let updated_user = Model {
    ///     id: user.id,
    ///     email: "anders@student.aau.dk".into(),
    ///     username: "andersAnden",
    ///     password: user.password
    /// }
    /// assert_eq!(context.update(updated_user).unwrap(),Model {
    ///     id: 1,
    ///     email: "anders@student.aau.dk".into(),
    ///     username: "andersAnden".into(),
    ///     password:"qwerty".into();
    /// }
    /// ```
    /// # Note
    /// The user entity's id will never be changed. If this behavior is wanted, delete the old user and create a new one.
    async fn update(&self, entity: user::Model) -> Result<user::Model, DbErr> {
        user::ActiveModel {
            id: Unchanged(entity.id),
            email: Set(entity.email),
            username: Set(entity.username),
            password: Set(entity.password),
        }
        .update(&self.db_context.get_connection())
        .await
    }

    /// Returns and deletes a user entity by id
    ///
    /// # Example
    /// ```
    /// let context : UserContext = UserContext::new(...);
    /// let user = context.get_by_id(1).unwrap();
    /// let deleted_user = Model {
    ///     id: user.id,
    ///     email: "anders@student.aau.dk".into(),
    ///     username: "andersAnden",
    ///     password: user.password
    /// }
    async fn delete(&self, entity_id: i32) -> Result<user::Model, DbErr> {
        let user = self
            .get_by_id(entity_id)
            .await?
            .ok_or(DbErr::RecordNotFound("No record was deleted".into()))?;
        user::Entity::delete_by_id(entity_id)
            .exec(&self.db_context.get_connection())
            .await
            .map(|_| user)
    }
}

#[cfg(test)]
mod tests {
    use crate::contexts::helpers::*;
    use crate::{
        contexts::UserContext,
        contexts::{EntityContextTrait, UserContextTrait},
        entities::{access, project, session, user},
        to_active_models,
    };
    use sea_orm::{entity::prelude::*, IntoActiveModel};
    use std::matches;

    async fn seed_db() -> (UserContext, user::Model) {
        let db_context = get_reset_database_context().await;

        let user_context = UserContext::new(db_context);

        let user = create_users(1)[0].clone();

        (user_context, user)
    }

    // Test the functionality of the 'create' function, which creates a user in the contexts
    #[tokio::test]
    async fn create_test() {
        // Setting up contexts and user context
        let (user_context, user) = seed_db().await;

        // Creates the user in the contexts using the 'create' function
        let created_user = user_context.create(user.clone()).await.unwrap();

        let fetched_user = user::Entity::find_by_id(created_user.id)
            .one(&user_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        // Assert if the new_user, created_user, and fetched_user are the same
        assert_eq!(user, created_user);
        assert_eq!(created_user, fetched_user);
    }

    #[tokio::test]
    async fn create_non_unique_username_test() {
        // Setting up contexts and user context
        let (user_context, user) = seed_db().await;

        // Creates a model of the user which will be created
        let mut users = create_users(2);

        users[0].username = user.clone().username;
        users[1].username = user.clone().username;

        // Creates the user in the contexts using the 'create' function
        let _created_user1 = user_context.create(users[0].clone()).await.unwrap();
        let created_user2 = user_context.create(users[1].clone()).await;

        assert!(matches!(
            created_user2.unwrap_err().sql_err(),
            Some(SqlErr::UniqueConstraintViolation(_))
        ));
    }

    #[tokio::test]
    async fn create_non_unique_email_test() {
        // Setting up contexts and user context
        let (user_context, user) = seed_db().await;

        // Creates a model of the user which will be created
        let mut users = create_users(2);

        users[0].email = user.clone().email;
        users[1].email = user.clone().email;

        // Creates the user in the contexts using the 'create' function
        let _created_user1 = user_context.create(users[0].clone()).await.unwrap();
        let created_user2 = user_context.create(users[1].clone()).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            created_user2.unwrap_err().sql_err(),
            Some(SqlErr::UniqueConstraintViolation(_))
        ));
    }

    #[tokio::test]
    async fn create_auto_increment_test() {
        // Setting up contexts and user context
        let (user_context, user) = seed_db().await;

        let mut users = create_users(2);

        users[0].id = user.clone().id;
        users[1].id = user.clone().id;

        // Creates the user in the contexts using the 'create' function
        let created_user1 = user_context.create(users[0].clone()).await.unwrap();
        let created_user2 = user_context.create(users[1].clone()).await.unwrap();

        let fetched_user1 = user::Entity::find_by_id(created_user1.id)
            .one(&user_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        let fetched_user2 = user::Entity::find_by_id(created_user2.id)
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
        // Setting up contexts and user context
        let (user_context, user) = seed_db().await;

        // Creates the user in the contexts using the 'create' function
        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        // Fetches the user created using the 'get_by_id' function
        let fetched_user = user_context.get_by_id(user.id).await.unwrap().unwrap();

        // Assert if the new_user, created_user, and fetched_user are the same
        assert_eq!(user, fetched_user);
    }

    #[tokio::test]
    async fn get_by_non_existing_id_test() {
        // Setting up contexts and user context
        let (user_context, _) = seed_db().await;

        // Fetches the user created using the 'get_by_id' function
        let fetched_user = user_context.get_by_id(1).await.unwrap();

        assert!(fetched_user.is_none());
    }

    #[tokio::test]
    async fn get_all_test() {
        // Setting up contexts and user context
        let (user_context, _) = seed_db().await;

        let users = create_users(10);
        let active_users_vec = to_active_models!(users.clone());

        user::Entity::insert_many(active_users_vec)
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(user_context.get_all().await.unwrap().len(), 10);

        let mut sorted = users.clone();
        sorted.sort_by_key(|k| k.id);

        for (i, user) in sorted.into_iter().enumerate() {
            assert_eq!(user, users[i]);
        }
    }

    #[tokio::test]
    async fn get_all_empty_test() {
        // Setting up contexts and user context
        let (user_context, _) = seed_db().await;

        let result = user_context.get_all().await.unwrap();
        let empty_users: Vec<user::Model> = vec![];

        assert_eq!(empty_users, result);
    }

    #[tokio::test]
    async fn update_test() {
        // Setting up contexts and user context
        let (user_context, user) = seed_db().await;

        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        let new_user = user::Model { ..user };

        let updated_user = user_context.update(new_user.clone()).await.unwrap();

        let fetched_user = user::Entity::find_by_id(updated_user.id)
            .one(&user_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(new_user, updated_user);
        assert_eq!(updated_user, fetched_user);
    }

    #[tokio::test]
    async fn update_modifies_username_test() {
        let (user_context, user) = seed_db().await;

        let user = user::Model {
            username: "tester1".into(),
            ..user.clone()
        };

        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        let new_user = user::Model {
            username: "tester2".into(),
            ..user.clone()
        };

        let updated_user = user_context.update(new_user.clone()).await.unwrap();

        assert_ne!(user, updated_user);
        assert_ne!(user, new_user);
    }

    #[tokio::test]
    async fn update_modifies_email_test() {
        let (user_context, user) = seed_db().await;

        let user = user::Model {
            email: "tester1@mail.dk".into(),
            ..user.clone()
        };

        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        let new_user = user::Model {
            email: "tester2@mail.dk".into(),
            ..user.clone()
        };

        let updated_user = user_context.update(new_user.clone()).await.unwrap();

        assert_ne!(user, updated_user);
        assert_ne!(user, new_user);
    }

    #[tokio::test]
    async fn update_modifies_password_test() {
        let (user_context, user) = seed_db().await;

        let user = user::Model {
            password: "12345".into(),
            ..user.clone()
        };

        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        let new_user = user::Model {
            password: "123456".into(),
            ..user.clone()
        };

        let updated_user = user_context.update(new_user.clone()).await.unwrap();

        assert_ne!(user, updated_user);
        assert_ne!(user, new_user);
    }

    #[tokio::test]
    async fn update_does_not_modify_id_test() {
        let (user_context, user) = seed_db().await;

        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        let updated_user = user::Model {
            id: user.id + 1,
            ..user
        };

        let res = user_context.update(updated_user.clone()).await;

        assert!(matches!(res.unwrap_err(), DbErr::RecordNotUpdated));
    }

    #[tokio::test]
    async fn update_non_unique_username_test() {
        // Setting up contexts and user context
        let (user_context, _) = seed_db().await;

        let users = create_users(2);

        user::Entity::insert_many(to_active_models!(users.clone()))
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        let new_user = user::Model {
            username: users[1].clone().username,
            ..users[0].clone()
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
        // Setting up contexts and user context
        let (user_context, _) = seed_db().await;

        // Creates a model of the user which will be created
        let users = create_users(2);

        user::Entity::insert_many(to_active_models!(users.clone()))
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        let new_user = user::Model {
            email: users[1].clone().email,
            ..users[0].clone()
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
        // Setting up contexts and user context
        let (user_context, user) = seed_db().await;

        let updated_user = user_context.update(user.clone()).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(updated_user.unwrap_err(), DbErr::RecordNotUpdated));
    }

    #[tokio::test]
    async fn delete_test() {
        // Setting up contexts and user context
        let (user_context, user) = seed_db().await;

        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        let deleted_user = user_context.delete(user.id).await.unwrap();

        let all_users = user::Entity::find()
            .all(&user_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(user, deleted_user);
        assert!(all_users.is_empty());
    }

    #[tokio::test]
    async fn delete_cascade_project_test() {
        // Setting up contexts and user context
        let (user_context, user) = seed_db().await;

        let project = create_projects(1, user.clone().id)[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();
        project::Entity::insert(project.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        user_context.delete(user.id).await.unwrap();

        let all_users = user::Entity::find()
            .all(&user_context.db_context.get_connection())
            .await
            .unwrap();
        let all_projects = project::Entity::find()
            .all(&user_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_users.len(), 0);
        assert_eq!(all_projects.len(), 0);
    }

    #[tokio::test]
    async fn delete_cascade_access_test() {
        // Setting up contexts and user context
        let (user_context, user) = seed_db().await;

        let project = create_projects(1, user.clone().id)[0].clone();
        let access = create_accesses(1, user.clone().id, project.clone().id)[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();
        project::Entity::insert(project.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();
        access::Entity::insert(access.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        user_context.delete(user.id).await.unwrap();

        let all_users = user::Entity::find()
            .all(&user_context.db_context.get_connection())
            .await
            .unwrap();
        let all_projects = project::Entity::find()
            .all(&user_context.db_context.get_connection())
            .await
            .unwrap();
        let all_accesses = access::Entity::find()
            .all(&user_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_users.len(), 0);
        assert_eq!(all_projects.len(), 0);
        assert_eq!(all_accesses.len(), 0);
    }

    #[tokio::test]
    async fn delete_cascade_session_test() {
        // Setting up contexts and user context
        let (user_context, user) = seed_db().await;

        let session = create_sessions(1, user.clone().id)[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();
        session::Entity::insert(session.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        user_context.delete(user.id).await.unwrap();

        let all_users = user::Entity::find()
            .all(&user_context.db_context.get_connection())
            .await
            .unwrap();
        let all_sessions = session::Entity::find()
            .all(&user_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(all_users.len(), 0);
        assert_eq!(all_sessions.len(), 0);
    }

    #[tokio::test]
    async fn delete_non_existing_id_test() {
        // Setting up contexts and user context
        let (user_context, _) = seed_db().await;

        let deleted_user = user_context.delete(1).await;

        // Assert if the new_user, created_user, and fetched_user are the same
        assert!(matches!(
            deleted_user.unwrap_err(),
            DbErr::RecordNotFound(_)
        ));
    }

    #[tokio::test]
    async fn get_by_username_test() {
        let (user_context, user) = seed_db().await;

        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        // Fetches the user created using the 'get_by_username' function
        let fetched_user = user_context
            .get_by_username(user.username.clone())
            .await
            .unwrap();

        // Assert if the fetched user is the same as the created user
        assert_eq!(fetched_user.unwrap().username, user.username);
    }

    #[tokio::test]
    async fn get_by_email_test() {
        let (user_context, user) = seed_db().await;

        user::Entity::insert(user.clone().into_active_model())
            .exec(&user_context.db_context.get_connection())
            .await
            .unwrap();

        let fetched_user = user_context.get_by_email(user.email.clone()).await.unwrap();

        assert_eq!(fetched_user.unwrap().email, user.email);
    }
}
