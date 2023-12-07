use crate::api::server::server::{
    CreateUserRequest, GetUsersRequest, GetUsersResponse, UpdateUserRequest,
};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

#[async_trait]
pub trait UserLogicTrait {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<()>, Status>;
    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<()>, Status>;
    async fn delete_user(&self, request: Request<()>) -> Result<Response<()>, Status>;
    async fn get_users(
        &self,
        request: Request<GetUsersRequest>,
    ) -> Result<Response<GetUsersResponse>, Status>;
}
