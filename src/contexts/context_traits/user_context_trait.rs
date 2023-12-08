use crate::contexts::context_traits::EntityContextTrait;
use crate::entities::user;
use async_trait::async_trait;
use sea_orm::DbErr;

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
