use crate::api::server::server::ecdar_backend_client::EcdarBackendClient;
use crate::api::server::server::ecdar_backend_server::EcdarBackend;
use crate::api::server::server::{
    QueryRequest, QueryResponse, SimulationStartRequest, SimulationStepRequest,
    SimulationStepResponse, UserTokenResponse,
};
use async_trait::async_trait;
use std::env;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct ReveaalContext;

impl ReveaalContext {
    async fn get_connection() -> EcdarBackendClient<Channel> {
        let url = env::var("REVEAAL_ADDRESS").expect("Expected REVEAAL_ADDRESS to be set.");
        EcdarBackendClient::connect(url).await.unwrap()
    }
}

#[async_trait]
impl EcdarBackend for ReveaalContext {
    async fn get_user_token(
        &self,
        request: Request<()>,
    ) -> Result<Response<UserTokenResponse>, Status> {
        Ok(ReveaalContext::get_connection()
            .await
            .get_user_token(request)
            .await
            .unwrap())
    }

    async fn send_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        Ok(ReveaalContext::get_connection()
            .await
            .send_query(request)
            .await
            .unwrap())
    }

    async fn start_simulation(
        &self,
        request: Request<SimulationStartRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        Ok(ReveaalContext::get_connection()
            .await
            .start_simulation(request)
            .await
            .unwrap())
    }

    async fn take_simulation_step(
        &self,
        request: Request<SimulationStepRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        Ok(ReveaalContext::get_connection()
            .await
            .take_simulation_step(request)
            .await
            .unwrap())
    }
}
