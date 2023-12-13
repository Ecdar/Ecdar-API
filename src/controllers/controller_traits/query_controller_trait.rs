use crate::api::server::protobuf::{
    CreateQueryRequest, DeleteQueryRequest, SendQueryRequest, SendQueryResponse, UpdateQueryRequest,
};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

#[async_trait]
pub trait QueryControllerTrait: Send + Sync {
    /// Creates a query in the contexts
    /// # Errors
    /// Returns an error if the contexts context fails to create the query or
    async fn create_query(
        &self,
        request: Request<CreateQueryRequest>,
    ) -> Result<Response<()>, Status>;

    /// Endpoint for updating a query record.
    /// # Errors
    /// Errors on non existent entity, parsing error or invalid rights
    async fn update_query(
        &self,
        request: Request<UpdateQueryRequest>,
    ) -> Result<Response<()>, Status>;

    /// Deletes a query record in the contexts.
    /// # Errors
    /// Returns an error if the provided query_id is not found in the contexts.
    async fn delete_query(
        &self,
        request: Request<DeleteQueryRequest>,
    ) -> Result<Response<()>, Status>;

    /// Sends a query to be run on Reveaal.
    /// After query is run the result is stored in the contexts.
    ///  
    /// Returns the response that is received from Reveaal.
    async fn send_query(
        &self,
        request: Request<SendQueryRequest>,
    ) -> Result<Response<SendQueryResponse>, Status>;
}
