use crate::controllers::controller_collection::ControllerCollection;

/// The collection of all controllers that Ecdar API offers.
#[derive(Clone)]
pub struct ConcreteEcdarApi {
    controllers: ControllerCollection,
}

impl ConcreteEcdarApi {
    pub fn new(controllers: ControllerCollection) -> Self {
        ConcreteEcdarApi { controllers }
    }
}

/// A module that contains all implementations for protobuffer endpoints.
///
/// The module uses the attribute macro `endpoints` to automatically implement the `endpoints` function as specified by the protobuffers.
/// Therefore, if new endpoints or services are added and implemented by the api server, then the macro will automatically add it to the list.
/// The macro can be found in the `ecdar_api_macros` crate.
#[ecdar_api_macros::endpoints]
mod routes {
    use super::super::server::protobuf::{
        ecdar_api_auth_server::EcdarApiAuth, ecdar_api_server::EcdarApi,
        ecdar_backend_server::EcdarBackend, CreateAccessRequest, CreateProjectRequest,
        CreateProjectResponse, CreateQueryRequest, CreateUserRequest, DeleteAccessRequest,
        DeleteProjectRequest, DeleteQueryRequest, EndpointsResponse, GetAuthTokenRequest,
        GetAuthTokenResponse, GetProjectRequest, GetProjectResponse, GetUsersRequest,
        GetUsersResponse, ListAccessInfoRequest, ListAccessInfoResponse, ListProjectsInfoResponse,
        QueryRequest, QueryResponse, SendQueryRequest, SendQueryResponse, SimulationStartRequest,
        SimulationStepRequest, SimulationStepResponse, UpdateAccessRequest, UpdateProjectRequest,
        UpdateQueryRequest, UpdateUserRequest, UserTokenResponse,
    };

    use tonic::{Request, Response, Status};

    /// Implementation of all the endpoits that the Ecdar API service expose through protobuffers.
    #[tonic::async_trait]
    impl EcdarApi for super::ConcreteEcdarApi {
        async fn get_project(
            &self,
            request: Request<GetProjectRequest>,
        ) -> Result<Response<GetProjectResponse>, Status> {
            self.controllers
                .project_controller
                .get_project(request)
                .await
        }

        async fn create_project(
            &self,
            request: Request<CreateProjectRequest>,
        ) -> Result<Response<CreateProjectResponse>, Status> {
            self.controllers
                .project_controller
                .create_project(request)
                .await
        }

        async fn update_project(
            &self,
            request: Request<UpdateProjectRequest>,
        ) -> Result<Response<()>, Status> {
            self.controllers
                .project_controller
                .update_project(request)
                .await
        }

        async fn delete_project(
            &self,
            request: Request<DeleteProjectRequest>,
        ) -> Result<Response<()>, Status> {
            self.controllers
                .project_controller
                .delete_project(request)
                .await
        }

        async fn list_projects_info(
            &self,
            request: Request<()>,
        ) -> Result<Response<ListProjectsInfoResponse>, Status> {
            self.controllers
                .project_controller
                .list_projects_info(request)
                .await
        }

        async fn list_access_info(
            &self,
            request: Request<ListAccessInfoRequest>,
        ) -> Result<Response<ListAccessInfoResponse>, Status> {
            self.controllers
                .access_controller
                .list_access_info(request)
                .await
        }

        async fn create_access(
            &self,
            request: Request<CreateAccessRequest>,
        ) -> Result<Response<()>, Status> {
            self.controllers
                .access_controller
                .create_access(request)
                .await
        }

        async fn update_access(
            &self,
            request: Request<UpdateAccessRequest>,
        ) -> Result<Response<()>, Status> {
            self.controllers
                .access_controller
                .update_access(request)
                .await
        }

        async fn delete_access(
            &self,
            request: Request<DeleteAccessRequest>,
        ) -> Result<Response<()>, Status> {
            self.controllers
                .access_controller
                .delete_access(request)
                .await
        }

        async fn update_user(
            &self,
            request: Request<UpdateUserRequest>,
        ) -> Result<Response<()>, Status> {
            self.controllers.user_controller.update_user(request).await
        }

        async fn delete_user(&self, request: Request<()>) -> Result<Response<()>, Status> {
            self.controllers.user_controller.delete_user(request).await
        }

        async fn get_users(
            &self,
            request: Request<GetUsersRequest>,
        ) -> Result<Response<GetUsersResponse>, Status> {
            self.controllers.user_controller.get_users(request).await
        }

        async fn create_query(
            &self,
            request: Request<CreateQueryRequest>,
        ) -> Result<Response<()>, Status> {
            self.controllers
                .query_controller
                .create_query(request)
                .await
        }

        async fn update_query(
            &self,
            request: Request<UpdateQueryRequest>,
        ) -> Result<Response<()>, Status> {
            self.controllers
                .query_controller
                .update_query(request)
                .await
        }

        async fn delete_query(
            &self,
            request: Request<DeleteQueryRequest>,
        ) -> Result<Response<()>, Status> {
            self.controllers
                .query_controller
                .delete_query(request)
                .await
        }

        async fn send_query(
            &self,
            request: Request<SendQueryRequest>,
        ) -> Result<Response<SendQueryResponse>, Status> {
            self.controllers.query_controller.send_query(request).await
        }

        async fn delete_session(&self, request: Request<()>) -> Result<Response<()>, Status> {
            self.controllers
                .session_controller
                .delete_session(request)
                .await
        }
    }

    /// Implementation of the EcdarBackend trait, which is used to ensure backwards compatability with the Reveaal engine.
    #[tonic::async_trait]
    impl EcdarBackend for super::ConcreteEcdarApi {
        async fn get_user_token(
            &self,
            _request: Request<()>,
        ) -> Result<Response<UserTokenResponse>, Status> {
            self.controllers
                .reveaal_controller
                .get_user_token(_request)
                .await
        }

        async fn send_query(
            &self,
            request: Request<QueryRequest>,
        ) -> Result<Response<QueryResponse>, Status> {
            self.controllers
                .reveaal_controller
                .send_query(request)
                .await
        }

        async fn start_simulation(
            &self,
            request: Request<SimulationStartRequest>,
        ) -> Result<Response<SimulationStepResponse>, Status> {
            self.controllers
                .reveaal_controller
                .start_simulation(request)
                .await
        }

        async fn take_simulation_step(
            &self,
            request: Request<SimulationStepRequest>,
        ) -> Result<Response<SimulationStepResponse>, Status> {
            self.controllers
                .reveaal_controller
                .take_simulation_step(request)
                .await
        }
    }

    /// The implementation for EcdarApiAuth.
    /// NOTE that this is the implementation that the macro extends.
    /// Therefore if changed then the macro should be changed too, else you will only get compile errors.
    #[tonic::async_trait]
    impl EcdarApiAuth for super::ConcreteEcdarApi {
        async fn get_auth_token(
            &self,
            request: Request<GetAuthTokenRequest>,
        ) -> Result<Response<GetAuthTokenResponse>, Status> {
            self.controllers
                .session_controller
                .get_auth_token(request)
                .await
        }

        async fn create_user(
            &self,
            request: Request<CreateUserRequest>,
        ) -> Result<Response<()>, Status> {
            self.controllers.user_controller.create_user(request).await
        }
    }
}
