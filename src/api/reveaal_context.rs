use crate::api::server::protobuf::ecdar_backend_client::EcdarBackendClient;
use crate::api::server::protobuf::ecdar_backend_server::EcdarBackend;
use crate::api::server::protobuf::{
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
    #[allow(clippy::expect_used)]
    async fn get_connection() -> Result<EcdarBackendClient<Channel>, tonic::transport::Error> {
        let url = env::var("REVEAAL_ADDRESS").expect("Expected REVEAAL_ADDRESS to be set.");
        EcdarBackendClient::connect(url).await
    }
}

#[async_trait]
impl EcdarBackend for ReveaalContext {
    async fn get_user_token(
        &self,
        request: Request<()>,
    ) -> Result<Response<UserTokenResponse>, Status> {
        ReveaalContext::get_connection()
            .await
            .map_err(|err| Status::internal(format!("{err}")))?
            .get_user_token(request)
            .await
    }

    async fn send_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        ReveaalContext::get_connection()
            .await
            .map_err(|err| Status::internal(format!("{err}")))?
            .send_query(request)
            .await
    }

    async fn start_simulation(
        &self,
        request: Request<SimulationStartRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        ReveaalContext::get_connection()
            .await
            .map_err(|err| Status::internal(format!("{err}")))?
            .start_simulation(request)
            .await
    }

    async fn take_simulation_step(
        &self,
        request: Request<SimulationStepRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        ReveaalContext::get_connection()
            .await
            .map_err(|err| Status::internal(format!("{err}")))?
            .take_simulation_step(request)
            .await
    }
}
