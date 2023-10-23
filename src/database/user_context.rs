use crate::database::database_context::DatabaseContext;
use crate::database::entity_context::EntityContextTrait;
use crate::entities::prelude::User;
use crate::entities::user::{ActiveModel, Model};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, RuntimeErr};
use std::future::Future;

pub struct UserContext {
    db_context: DatabaseContext,
}

#[async_trait]
pub trait UserContextTrait {}

impl UserContextTrait for UserContext {}

#[async_trait]
impl EntityContextTrait<Model> for UserContext {
    fn new(db_context: DatabaseContext) -> Self {
        UserContext { db_context }
    }

    /// Used for creating a User entity
    /// # Example
    /// ```
    /// let model : Model = {
    ///     id: 1,
    ///     email: "anders@aau.dk".into(),
    ///     username: "Anders".into(),
    ///     password: "qwerty".into()
    /// }
    /// let context : UserContext = UserContext::new(...);
    /// context.create(model);
    /// ```
    async fn create(&self, entity: Model) -> Result<Model, DbErr> {
        let user = ActiveModel {
            id: Default::default(),
            email: Set(entity.email),
            username: Set(entity.username),
            password: Set(entity.password),
        };

        let user: Model = user.insert(&self.db_context.db).await?;
        Ok(user)
    }

    /// Returns a single user entity (uses primary key)
    /// # Example
    /// ```
    /// let context : UserContext = UserContext::new(...);
    /// let model : Model = context.get_by_id(1).unwrap();
    /// assert_eq!(model.username,"Anders".into());
    /// ```
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<Model>, DbErr> {
        User::find_by_id(entity_id).one(&self.db_context.db).await
    }

    /// Returns all the user entities
    /// # Example
    /// ```
    /// let context : UserContext = UserContext::new(...);
    /// let model : vec<Model> = context.get_all().unwrap();
    /// assert_eq!(model.len(),1);
    /// ```
    async fn get_all(&self) -> Result<Vec<Model>, DbErr> {
        User::find().all(&self.db_context.db).await
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
    /// The user entity's id will never be changed. If this behavior is wanted, delete the old user and create a one.
    async fn update(&self, entity: Model) -> Result<Model, DbErr> {
        let res = &self.get_by_id(entity.id).await?;
        let updated_user: Result<Model, DbErr> = match res {
            None => Err(DbErr::RecordNotFound(format!(
                "Could not find entity {:?}",
                entity
            ))),
            Some(user) => {
                ActiveModel {
                    id: Unchanged(user.id), //TODO ved ikke om unchanged betyder det jeg tror det betyder
                    email: Set(entity.email),
                    username: Set(entity.username),
                    password: Set(entity.password),
                }
                .update(&self.db_context.db)
                .await
            }
        };
        return updated_user;
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
    async fn delete(&self, entity_id: i32) -> Result<Model, DbErr> {
        let user = self.get_by_id(entity_id).await?;
        match user {
            None => Err(DbErr::Exec(RuntimeErr::Internal(
                "No record was deleted".into(),
            ))),
            Some(user) => {
                User::delete_by_id(entity_id)
                    .exec(&self.db_context.db)
                    .await?;
                Ok(user)
            }
        }
    }
}
#[cfg(test)]
#[path = "../tests/database/user_context.rs"]
mod tests;
