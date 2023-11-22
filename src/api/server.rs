use std::env;
use std::sync::Arc;

use tonic::transport::Server;

use crate::api::auth;
use crate::api::ecdar_api::ConcreteEcdarApi;
use crate::api::server::server::ecdar_api_auth_server::EcdarApiAuthServer;
use crate::api::server::server::ecdar_api_server::EcdarApiServer;
use crate::api::server::server::ecdar_backend_server::{EcdarBackend, EcdarBackendServer};
use crate::database::access_context::AccessContextTrait;
use crate::database::in_use_context::InUseContextTrait;
use crate::database::model_context::ModelContextTrait;
use crate::database::query_context::QueryContextTrait;
use crate::database::session_context::SessionContextTrait;
use crate::database::user_context::UserContextTrait;

pub mod server {
    tonic::include_proto!("ecdar_proto_buf");
}

pub async fn start_grpc_server(
    access_context: Arc<dyn AccessContextTrait>,
    in_use_context: Arc<dyn InUseContextTrait>,
    model_context: Arc<dyn ModelContextTrait>,
    query_context: Arc<dyn QueryContextTrait>,
    session_context: Arc<dyn SessionContextTrait>,
    user_context: Arc<dyn UserContextTrait>,
    reveaal_context: Arc<dyn EcdarBackend>,
) -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = env::var("API_ADDRESS")
        .expect("Expected API_ADDRESS to be set.")
        .parse()
        .unwrap();

    println!("Starting grpc server on '{}'", addr);

    let svc = ConcreteEcdarApi::new(
        access_context,
        in_use_context,
        model_context,
        query_context,
        session_context,
        user_context,
        reveaal_context,
    )
    .await;

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
