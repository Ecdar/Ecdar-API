use crate::api::server::server::ecdar_backend_client::EcdarBackendClient;
use crate::api::server::server::{
    QueryRequest, QueryResponse, SimulationStartRequest, SimulationStepRequest,
    SimulationStepResponse, UserTokenResponse,
};
use crate::logics::logic_impls::ReveaalLogic;
use crate::services::service_traits::ReveaalServiceTrait;
use std::env;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

pub struct ReveaalService;

impl ReveaalLogic {
    async fn get_connection() -> EcdarBackendClient<Channel> {
        let url = env::var("REVEAAL_ADDRESS").expect("Expected REVEAAL_ADDRESS to be set.");
        EcdarBackendClient::connect(url).await.unwrap()
    }
}

impl ReveaalServiceTrait for ReveaalService {
    async fn get_user_token(
        &self,
        request: Request<()>,
    ) -> Result<Response<UserTokenResponse>, Status> {
        Ok(ReveaalLogic::get_connection()
            .await
            .get_user_token(request)
            .await
            .unwrap())
    }

    async fn send_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        Ok(ReveaalLogic::get_connection()
            .await
            .send_query(request)
            .await
            .unwrap())
    }

    async fn start_simulation(
        &self,
        request: Request<SimulationStartRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        Ok(ReveaalLogic::get_connection()
            .await
            .start_simulation(request)
            .await
            .unwrap())
    }

    async fn take_simulation_step(
        &self,
        request: Request<SimulationStepRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        Ok(ReveaalLogic::get_connection()
            .await
            .take_simulation_step(request)
            .await
            .unwrap())
    }
}
