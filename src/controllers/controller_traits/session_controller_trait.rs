use crate::api::server::protobuf::{GetAuthTokenRequest, GetAuthTokenResponse};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

#[async_trait]
pub trait SessionControllerTrait: Send + Sync {
    /// Deletes the requester's session, found by their access token.
    ///  
    /// Returns the response that is received from Reveaal.
    async fn delete_session(&self, _request: Request<()>) -> Result<Response<()>, Status>;

    /// This method is used to get a new access and refresh token for a user.
    ///
    /// # Errors
    /// This function will return an error if the user does not exist in the contexts,
    /// if the password in the request does not match the user's password,
    /// or if no user is provided in the request.
    async fn get_auth_token(
        &self,
        request: Request<GetAuthTokenRequest>,
    ) -> Result<Response<GetAuthTokenResponse>, Status>;
}
