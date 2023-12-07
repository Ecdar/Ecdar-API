use crate::api::server::server::{GetAuthTokenRequest, GetAuthTokenResponse};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

#[async_trait]
pub trait SessionControllerTrait {
    async fn delete_session(&self, _request: Request<()>) -> Result<Response<()>, Status>;
    async fn get_auth_token(
        &self,
        request: Request<GetAuthTokenRequest>,
    ) -> Result<Response<GetAuthTokenResponse>, Status>;
}
