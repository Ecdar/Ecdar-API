use std::env;
use crate::api_server::ecdar_api::ConcreteEcdarApi;
use crate::api_server::server::server::ecdar_backend_server::EcdarBackendServer;
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

    // creating a service
    println!("Starting grpc server on '{}'", addr);

    // adding our service to our server.
    Server::builder()
        .add_service(EcdarBackendServer::new(ConcreteEcdarApi::new()))
        .serve(addr)
        .await?;
    Ok(())
}
