use crate::api::auth::TokenType;
use crate::database::context_traits::EntityContextTrait;
use crate::entities::session;
use async_trait::async_trait;
use sea_orm::DbErr;

#[async_trait]
pub trait SessionContextTrait: EntityContextTrait<session::Model> {
    async fn get_by_token(
        &self,
        token_type: TokenType,
        token: String,
    ) -> Result<Option<session::Model>, DbErr>;

    async fn delete_by_token(
        &self,
        token_type: TokenType,
        token: String,
    ) -> Result<session::Model, DbErr>;
}
