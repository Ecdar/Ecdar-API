use crate::api::auth;
use crate::api::ecdar_api::ConcreteEcdarApi;
use crate::api::server::server::ecdar_api_auth_server::EcdarApiAuthServer;
use crate::api::server::server::ecdar_api_server::EcdarApiServer;
use crate::api::server::server::ecdar_backend_server::EcdarBackendServer;
use std::env;
use tonic::transport::Server;

pub mod server {
    tonic::include_proto!("ecdar_proto_buf");
}

pub async fn start_grpc_server() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = env::var("API_ADDRESS")
        .expect("Expected API_ADDRESS to be set.")
        .parse()
        .unwrap();

    println!("Starting grpc server on '{}'", addr);

    // adding services to our server.
    Server::builder()
        .add_service(EcdarApiAuthServer::new(ConcreteEcdarApi::new()))
        .add_service(EcdarApiServer::with_interceptor(
            ConcreteEcdarApi::new(),
            auth::token_validation,
        ))
        .add_service(EcdarBackendServer::new(ConcreteEcdarApi::new()))
        .serve(addr)
        .await?;
    Ok(())
}
