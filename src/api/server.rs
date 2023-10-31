use crate::api::auth;
use crate::api::ecdar_api::ConcreteEcdarApi;
use crate::api::server::server::ecdar_api_auth_server::EcdarApiAuthServer;
use crate::api::server::server::ecdar_api_server::EcdarApiServer;
use crate::api::server::server::ecdar_backend_server::EcdarBackendServer;
use crate::database::database_context::{DatabaseContext, DatabaseContextTrait};
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

    let db_context = Box::new(DatabaseContext::new().await?);

    // adding services to our server.
    Server::builder()
        .add_service(EcdarApiAuthServer::new(
            ConcreteEcdarApi::new(db_context.clone()).await,
        ))
        .add_service(EcdarApiServer::with_interceptor(
            ConcreteEcdarApi::new(db_context.clone()).await,
            auth::token_validation,
        ))
        .add_service(EcdarBackendServer::new(
            ConcreteEcdarApi::new(db_context.clone()).await,
        ))
        .serve(addr)
        .await?;
    Ok(())
}
