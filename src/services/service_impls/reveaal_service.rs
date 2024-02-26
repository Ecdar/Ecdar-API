use crate::api::server::protobuf::ecdar_backend_client::EcdarBackendClient;
use crate::api::server::protobuf::{
    QueryRequest, QueryResponse, SimulationStartRequest, SimulationStepRequest,
    SimulationStepResponse, UserTokenResponse,
};
use crate::services::service_traits::ReveaalServiceTrait;
use async_trait::async_trait;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

pub struct ReveaalService {
    address: String,
}

impl ReveaalService {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
        }
    }

    async fn get_connection(&self) -> Result<EcdarBackendClient<Channel>, Status> {
        EcdarBackendClient::connect(self.address.clone())
            .await
            .map_err(|err| Status::internal(format!("{err}")))
    }
}

#[async_trait]
impl ReveaalServiceTrait for ReveaalService {
    async fn get_user_token(
        &self,
        request: Request<()>,
    ) -> Result<Response<UserTokenResponse>, Status> {
        self.get_connection().await?.get_user_token(request).await
    }

    async fn send_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        self.get_connection().await?.send_query(request).await
    }

    async fn start_simulation(
        &self,
        request: Request<SimulationStartRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        self.get_connection().await?.start_simulation(request).await
    }

    async fn take_simulation_step(
        &self,
        request: Request<SimulationStepRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        self.get_connection()
            .await?
            .take_simulation_step(request)
            .await
    }
}

#[cfg(test)]
mod tests {
    // use crate::api::server::server::QueryResponse;
    // use wiremock_grpc::generate;
    // use wiremock_grpc::*;
    //
    // generate!("EcdarBackend", MyMockServer);

    #[ignore]
    #[tokio::test]
    async fn send_query_test_correct_query_returns_ok() {
        //todo!("Somehow QueryResponse does not implement prost::message::Message even though it does.
        // supposedly a versioning error between wiremock_grpc, tonic, and prost")

        // let mut server = MyMockServer::start_default().await;
        //
        // let request1 = server.setup(
        //     MockBuilder::when()
        //         .path("EcdarBackend/SendQuery")
        //         .then()
        //         .return_status(Code::Ok)
        //         .return_body(|| QueryResponse {
        //             query_id: 0,
        //             info: vec![],
        //             result: None,
        //         }),
        // );

        //...
        //https://crates.io/crates/wiremock-grpc
    }
}
