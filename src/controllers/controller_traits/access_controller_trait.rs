use crate::api::server::protobuf::{
    CreateAccessRequest, DeleteAccessRequest, ListAccessInfoRequest, ListAccessInfoResponse,
    UpdateAccessRequest,
};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

#[async_trait]
pub trait AccessControllerTrait: Send + Sync {
    /// handles the list_access_info endpoint
    /// # Errors 
    /// If an invalid or non-existent [`ListAccessInfoRequest::project_id`] is provided
    async fn list_access_info(
        &self,
        request: Request<ListAccessInfoRequest>,
    ) -> Result<Response<ListAccessInfoResponse>, Status>;
    /// Creates an access in the contexts.
    /// # Errors
    /// Returns an error if the contexts context fails to create the access
    async fn create_access(
        &self,
        request: Request<CreateAccessRequest>,
    ) -> Result<Response<()>, Status>;

    /// Endpoint for updating an access record.
    ///
    /// Takes [`UpdateAccessRequest`] as input
    ///
    /// Returns a [`Status`] as response
    ///
    /// `project_id` and `user_id` is set to 'default' since they won't be updated in the contexts.
    async fn update_access(
        &self,
        request: Request<UpdateAccessRequest>,
    ) -> Result<Response<()>, Status>;
    
    /// Deletes the an Access from the contexts. This has no sideeffects.
    ///
    /// # Errors
    /// This function will return an error if the access does not exist in the contexts.
    async fn delete_access(
        &self,
        request: Request<DeleteAccessRequest>,
    ) -> Result<Response<()>, Status>;
}
