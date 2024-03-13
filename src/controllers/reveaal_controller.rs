use crate::api::server::protobuf::{
    ecdar_backend_server::EcdarBackend, QueryRequest, QueryResponse, SimulationStartRequest,
    SimulationStepRequest, SimulationStepResponse, UserTokenResponse,
};
use crate::services::ServiceCollection;
use async_trait::async_trait;
use tonic::{Request, Response, Status};

pub struct ReveaalController {
    services: ServiceCollection,
}

impl ReveaalController {
    pub fn new(services: ServiceCollection) -> Self {
        Self { services }
    }
}

#[async_trait]
impl EcdarBackend for ReveaalController {
    async fn get_user_token(
        &self,
        request: Request<()>,
    ) -> Result<Response<UserTokenResponse>, Status> {
        self.services.reveaal_service.get_user_token(request).await
    }
    /// Forwards the query to the Reveaal backend
    /// # Errors
    /// Errors when Reveaal backend errors
    async fn send_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        self.services.reveaal_service.send_query(request).await
    }
    /// Forwards the query to the Reveaal backend
    /// # Errors
    /// Errors when Reveaal backend errors
    //TODO Bedre doc comment
    async fn start_simulation(
        &self,
        request: Request<SimulationStartRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        self.services
            .reveaal_service
            .start_simulation(request)
            .await
    }
    //TODO Bedre doc comment
    async fn take_simulation_step(
        &self,
        request: Request<SimulationStepRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        self.services
            .reveaal_service
            .take_simulation_step(request)
            .await
    }
}
