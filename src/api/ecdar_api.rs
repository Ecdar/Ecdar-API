use super::server::server::{
    ecdar_api_auth_server::EcdarApiAuth, ecdar_api_server::EcdarApi,
    ecdar_backend_server::EcdarBackend, CreateAccessRequest, CreateProjectRequest,
    CreateProjectResponse, CreateQueryRequest, CreateUserRequest, DeleteAccessRequest,
    DeleteProjectRequest, DeleteQueryRequest, GetAuthTokenRequest, GetAuthTokenResponse,
    GetProjectRequest, GetProjectResponse, GetUsersRequest, GetUsersResponse,
    ListAccessInfoRequest, ListAccessInfoResponse, ListProjectsInfoResponse, QueryRequest,
    QueryResponse, SendQueryRequest, SendQueryResponse, SimulationStartRequest,
    SimulationStepRequest, SimulationStepResponse, UpdateAccessRequest, UpdateProjectRequest,
    UpdateQueryRequest, UpdateUserRequest, UserTokenResponse,
};
use crate::api::logic_collection::LogicCollection;
use serde_json;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct ConcreteEcdarApi {
    logics: LogicCollection,
}

impl ConcreteEcdarApi {
    pub fn new(logics: LogicCollection) -> Self {
        ConcreteEcdarApi { logics }
    }
}

#[tonic::async_trait]
impl EcdarApi for ConcreteEcdarApi {
    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<GetProjectResponse>, Status> {
        self.logics.project_logic.get_project(request).await
    }

    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
        self.logics.project_logic.create_project(request).await
    }

    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<()>, Status> {
        self.logics.project_logic.update_project(request).await
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<()>, Status> {
        self.logics.project_logic.delete_project(request).await
    }

    async fn list_projects_info(
        &self,
        request: Request<()>,
    ) -> Result<Response<ListProjectsInfoResponse>, Status> {
        self.logics.project_logic.list_projects_info(request).await
    }

    async fn list_access_info(
        &self,
        request: Request<ListAccessInfoRequest>,
    ) -> Result<Response<ListAccessInfoResponse>, Status> {
        self.logics.access_logic.list_access_info(request).await
    }

    async fn create_access(
        &self,
        request: Request<CreateAccessRequest>,
    ) -> Result<Response<()>, Status> {
        self.logics.access_logic.create_access(request).await
    }

    async fn update_access(
        &self,
        request: Request<UpdateAccessRequest>,
    ) -> Result<Response<()>, Status> {
        self.logics.access_logic.update_access(request).await
    }

    async fn delete_access(
        &self,
        request: Request<DeleteAccessRequest>,
    ) -> Result<Response<()>, Status> {
        self.logics.access_logic.delete_access(request).await
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<()>, Status> {
        self.logics.user_logic.update_user(request).await
    }

    async fn delete_user(&self, request: Request<()>) -> Result<Response<()>, Status> {
        self.logics.user_logic.delete_user(request).await
    }

    async fn get_users(
        &self,
        request: Request<GetUsersRequest>,
    ) -> Result<Response<GetUsersResponse>, Status> {
        self.logics.user_logic.get_users(request).await
    }

    async fn create_query(
        &self,
        request: Request<CreateQueryRequest>,
    ) -> Result<Response<()>, Status> {
        self.logics.query_logic.create_query(request).await
    }

    async fn update_query(
        &self,
        request: Request<UpdateQueryRequest>,
    ) -> Result<Response<()>, Status> {
        self.logics.query_logic.update_query(request).await
    }

    async fn delete_query(
        &self,
        request: Request<DeleteQueryRequest>,
    ) -> Result<Response<()>, Status> {
        self.logics.query_logic.delete_query(request).await
    }

    async fn send_query(
        &self,
        request: Request<SendQueryRequest>,
    ) -> Result<Response<SendQueryResponse>, Status> {
        self.logics.query_logic.send_query(request).await
    }

    async fn delete_session(&self, request: Request<()>) -> Result<Response<()>, Status> {
        self.logics.session_logic.delete_session(request).await
    }
}

/// Implementation of the EcdarBackend trait, which is used to ensure backwards compatability with the Reveaal engine.
#[tonic::async_trait]
impl EcdarBackend for ConcreteEcdarApi {
    async fn get_user_token(
        &self,
        _request: Request<()>,
    ) -> Result<Response<UserTokenResponse>, Status> {
        self.logics.reveaal_logic.get_user_token(_request).await
    }

    async fn send_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        self.logics.reveaal_logic.send_query(request).await
    }

    async fn start_simulation(
        &self,
        request: Request<SimulationStartRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        self.logics.reveaal_logic.start_simulation(request).await
    }

    async fn take_simulation_step(
        &self,
        request: Request<SimulationStepRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        self.logics
            .reveaal_logic
            .take_simulation_step(request)
            .await
    }
}

/// Implementation of the EcdarBackend trait, which is used to ensure backwards compatability with the Reveaal engine.
#[tonic::async_trait]
impl EcdarApiAuth for ConcreteEcdarApi {
    async fn get_auth_token(
        &self,
        request: Request<GetAuthTokenRequest>,
    ) -> Result<Response<GetAuthTokenResponse>, Status> {
        self.logics.session_logic.get_auth_token(request).await
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<()>, Status> {
        self.logics.user_logic.create_user(request).await
    }
}
