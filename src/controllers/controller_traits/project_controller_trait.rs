use crate::api::server::server::{
    CreateProjectRequest, CreateProjectResponse, DeleteProjectRequest, GetProjectRequest,
    GetProjectResponse, ListProjectsInfoResponse, UpdateProjectRequest,
};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

#[async_trait]
pub trait ProjectControllerTrait: Send + Sync {
    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<GetProjectResponse>, Status>;

    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status>;

    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<()>, Status>;

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<()>, Status>;

    async fn list_projects_info(
        &self,
        request: Request<()>,
    ) -> Result<Response<ListProjectsInfoResponse>, Status>;
}
