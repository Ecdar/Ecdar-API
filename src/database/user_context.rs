use crate::database::database_context::DatabaseContextTrait;
use crate::database::entity_context::EntityContextTrait;
use crate::entities::user;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};
use std::sync::Arc;

pub struct UserContext {
    db_context: Arc<dyn DatabaseContextTrait>,
}

#[async_trait]
pub trait UserContextTrait: EntityContextTrait<user::Model> {
    async fn get_by_username(&self, username: String) -> Result<Option<user::Model>, DbErr>;
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
    /// let model : Model = {
    ///     id: Default::default(),
    ///     email: "anders@aau.dk".into(),
    ///     username: "Anders".into(),
    ///     password: "qwerty".into()
    /// }
    /// let context : UserContext = UserContext::new(...);
    /// context.create(model);
    /// ```
    async fn create(&self, entity: user::Model) -> Result<user::Model, DbErr> {
        let user = user::ActiveModel {
            id: Default::default(),
            email: Set(entity.email),
            username: Set(entity.username),
            password: Set(entity.password),
        };
        let user = user.insert(&self.db_context.get_connection()).await?;
        Ok(user)
    }

    /// Returns a single user entity (uses primary key)
    /// # Example
    /// ```
    /// let context : UserContext = UserContext::new(...);
    /// let model : Model = context.get_by_id(1).unwrap();
    /// assert_eq!(model.username,"Anders".into());
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
    /// let model : vec<Model> = context.get_all().unwrap();
    /// assert_eq!(model.len(),1);
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
        let user = self.get_by_id(entity_id).await?;
        match user {
            None => Err(DbErr::RecordNotFound("No record was deleted".into())),
            Some(user) => {
                user::Entity::delete_by_id(entity_id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(user)
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/database/user_context.rs"]
mod user_context_tests;
