use std::env;

use tonic::transport::Server;

use crate::api::auth;
use crate::api::ecdar_api::ConcreteEcdarApi;
use crate::api::server::server::ecdar_api_auth_server::EcdarApiAuthServer;
use crate::api::server::server::ecdar_api_server::EcdarApiServer;
use crate::api::server::server::ecdar_backend_server::EcdarBackendServer;
use crate::controllers::controller_collection::ControllerCollection;

pub mod server {
    tonic::include_proto!("ecdar_proto_buf");
}

pub async fn start_grpc_server(
    controllers: ControllerCollection,
) -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = env::var("API_ADDRESS")
        .expect("Expected API_ADDRESS to be set.")
        .parse()
        .unwrap();

    println!("Starting grpc server on '{}'", addr);

    let svc = ConcreteEcdarApi::new(controllers);

    // adding services to our server.
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

