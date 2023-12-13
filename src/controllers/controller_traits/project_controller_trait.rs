use crate::api::server::protobuf::{
    CreateProjectRequest, CreateProjectResponse, DeleteProjectRequest, GetProjectRequest,
    GetProjectResponse, ListProjectsInfoResponse, UpdateProjectRequest,
};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

#[async_trait]
pub trait ProjectControllerTrait: Send + Sync {
    /// Gets a project and its queries from the contexts.
    ///
    /// If the project is not in use, it will now be in use by the requester's session,
    /// given that they are an Editor.
    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<GetProjectResponse>, Status>;

    /// Creates a project from [`CreateProjectRequest`]
    /// # Errors
    /// Errors on invalid JSON, invalid user id or if a project already exists
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status>;

    /// Updates a Model in the contexts given its id.
    ///
    /// # Errors
    /// This function will return an error if the project does not exist in the contexts
    /// or if the user does not have access to the project with role 'Editor'.
    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<()>, Status>;

    /// Deletes a Model from the contexts.
    ///
    /// # Errors
    /// This function will return an error if the project does not exist in the contexts
    /// or if the user is not the project owner.
    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<()>, Status>;

    async fn list_projects_info(
        &self,
        request: Request<()>,
    ) -> Result<Response<ListProjectsInfoResponse>, Status>;
}
