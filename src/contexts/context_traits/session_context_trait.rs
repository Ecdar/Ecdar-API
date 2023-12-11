use crate::api::auth::TokenType;
use crate::contexts::context_traits::EntityContextTrait;
use crate::entities::session;
use async_trait::async_trait;
use sea_orm::DbErr;

#[async_trait]
pub trait SessionContextTrait: EntityContextTrait<session::Model> {
    /// Searches for a token by `Access` or `Refresh` token,
    /// returning [`Some`] if one is found, [`None`] otherwise
    /// # Errors
    /// Errors on failed connection, execution error or constraint violations.
    async fn get_by_token(
        &self,
        token_type: TokenType,
        token: String,
    ) -> Result<Option<session::Model>, DbErr>;
    /// Searches for a token by `Access` or `Refresh` token, deleting and returning it
    /// # Errors
    /// Errors on failed connection, execution error or constraint violations.
    async fn delete_by_token(
        &self,
        token_type: TokenType,
        token: String,
    ) -> Result<session::Model, DbErr>;
}
