use crate::api::server::server::{
    CreateQueryRequest, DeleteQueryRequest, SendQueryRequest, SendQueryResponse, UpdateQueryRequest,
};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

#[async_trait]
pub trait QueryControllerTrait {
    async fn create_query(
        &self,
        request: Request<CreateQueryRequest>,
    ) -> Result<Response<()>, Status>;
    async fn update_query(
        &self,
        request: Request<UpdateQueryRequest>,
    ) -> Result<Response<()>, Status>;
    async fn delete_query(
        &self,
        request: Request<DeleteQueryRequest>,
    ) -> Result<Response<()>, Status>;
    async fn send_query(
        &self,
        request: Request<SendQueryRequest>,
    ) -> Result<Response<SendQueryResponse>, Status>;
}
