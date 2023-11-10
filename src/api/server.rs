use std::env;
use std::sync::Arc;

use tonic::transport::Server;

use crate::api::auth;
use crate::api::ecdar_api::ConcreteEcdarApi;
use crate::api::server::server::ecdar_api_auth_server::EcdarApiAuthServer;
use crate::api::server::server::ecdar_api_server::EcdarApiServer;
use crate::api::server::server::ecdar_backend_server::EcdarBackendServer;
use crate::database::access_context::{AccessContext, AccessContextTrait};
use crate::database::database_context::{DatabaseContext, DatabaseContextTrait};
use crate::database::entity_context::EntityContextTrait;
use crate::database::in_use_context::{InUseContext, InUseContextTrait};
use crate::database::model_context::{ModelContext, ModelContextTrait};
use crate::database::query_context::{QueryContext, QueryContextTrait};
use crate::database::session_context::{SessionContext, SessionContextTrait};
use crate::database::user_context::{UserContext, UserContextTrait};

pub mod server {
    tonic::include_proto!("ecdar_proto_buf");
}

pub async fn start_grpc_server(
    model_context: Arc<dyn ModelContextTrait + Send + Sync>,
    user_context: Arc<dyn UserContextTrait + Send + Sync>,
    access_context: Arc<dyn AccessContextTrait + Send + Sync>,
    query_context: Arc<dyn QueryContextTrait + Send + Sync>,
    session_context: Arc<dyn SessionContextTrait + Send + Sync>,
    in_use_context: Arc<dyn InUseContextTrait + Send + Sync>,
) -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = env::var("API_ADDRESS")
        .expect("Expected API_ADDRESS to be set.")
        .parse()
        .unwrap();

    println!("Starting grpc server on '{}'", addr);

    let svc = ConcreteEcdarApi::new(
        model_context,
        user_context,
        access_context,
        query_context,
        session_context,
        in_use_context,
    )
    .await;

    // adding services to our server.
    Server::builder()
        .add_service(EcdarApiAuthServer::new(svc.clone()))
        .add_service(EcdarApiServer::with_interceptor(
            svc.clone(),
            auth::token_validation,
        ))
        .add_service(EcdarBackendServer::new(svc.clone()))
        .serve(addr)
        .await?;
    Ok(())
}
