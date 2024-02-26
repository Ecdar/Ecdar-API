use std::env;

use tonic::transport::Server;

use crate::api::auth;
use crate::api::ecdar_api::ConcreteEcdarApi;
use crate::api::server::protobuf::ecdar_api_auth_server::EcdarApiAuthServer;
use crate::api::server::protobuf::ecdar_api_server::EcdarApiServer;
use crate::api::server::protobuf::ecdar_backend_server::EcdarBackendServer;
use crate::controllers::ControllerCollection;

pub mod protobuf {
    tonic::include_proto!("ecdar_proto_buf");
}
#[allow(clippy::expect_used)]
pub async fn start_grpc_server(
    controllers: ControllerCollection,
) -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = env::var("API_ADDRESS")
        .expect("Expected API_ADDRESS to be set.")
        .parse()
        .expect("failed to parse ip address from environment variable");

    println!("Starting grpc protobuf on '{}'", addr);

    let svc = ConcreteEcdarApi::new(controllers);

    // adding services to our protobuf.
    Server::builder()
        .add_service(EcdarApiAuthServer::new(svc.clone()))
        .add_service(EcdarApiServer::with_interceptor(
            svc.clone(),
            auth::validation_interceptor,
        ))
        .add_service(EcdarBackendServer::new(svc.clone()))
        .serve(addr)
        .await?;
    Ok(())
}
