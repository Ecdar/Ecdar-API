use crate::api::collections::ServiceCollection;
use crate::api::server::server::ecdar_backend_server::EcdarBackend;
use crate::api::server::server::{
    QueryRequest, QueryResponse, SimulationStartRequest, SimulationStepRequest,
    SimulationStepResponse, UserTokenResponse,
};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct ReveaalController {
    services: ServiceCollection,
}

#[async_trait]
impl EcdarBackend for ReveaalController {
    async fn get_user_token(
        &self,
        request: Request<()>,
    ) -> Result<Response<UserTokenResponse>, Status> {
        self.services.reveaal_service.get_user_token(request)
    }

    async fn send_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        self.services.reveaal_service.send_query(request)
    }

    async fn start_simulation(
        &self,
        request: Request<SimulationStartRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        self.services.reveaal_service.start_simulation(request)
    }

    async fn take_simulation_step(
        &self,
        request: Request<SimulationStepRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        self.services.reveaal_service.take_simulation_step(request)
    }
}
