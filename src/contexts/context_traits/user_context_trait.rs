use crate::contexts::context_traits::EntityContextTrait;
use crate::entities::user;
use async_trait::async_trait;
use sea_orm::DbErr;

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
