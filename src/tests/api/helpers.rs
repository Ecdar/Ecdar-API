#![cfg(test)]

use crate::api::ecdar_api::ConcreteEcdarApi;
use crate::api::server::server::ecdar_backend_server::EcdarBackend;
use crate::api::server::server::{
    QueryRequest, QueryResponse, SimulationStartRequest, SimulationStepRequest,
    SimulationStepResponse, UserTokenResponse,
};
use crate::database::access_context::AccessContext;
use crate::database::database_context::{
    DatabaseContextTrait, PostgresDatabaseContext, SQLiteDatabaseContext,
};
use crate::database::entity_context::EntityContextTrait;
use crate::database::in_use_context::InUseContext;
use crate::database::model_context::ModelContext;
use crate::database::query_context::QueryContext;
use crate::database::session_context::SessionContext;
use crate::database::user_context::UserContext;
use dotenv::dotenv;
use sea_orm::{ConnectionTrait, Database, DbBackend};
use std::env;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn get_reset_concrete_ecdar_api(
    mock_ecdar_backend: Arc<dyn EcdarBackend>,
) -> ConcreteEcdarApi {
    dotenv().ok();

    let url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests.");
    let db = Database::connect(&url).await.unwrap();
    let db_context: Arc<dyn DatabaseContextTrait> = match db.get_database_backend() {
        DbBackend::Sqlite => Arc::new(SQLiteDatabaseContext::new(&url).await.unwrap()),
        DbBackend::Postgres => Arc::new(PostgresDatabaseContext::new(&url).await.unwrap()),
        _ => panic!("Database protocol not supported"),
    };

    db_context.reset().await.unwrap();

    let model_context = Arc::new(ModelContext::new(db_context.clone()));
    let user_context = Arc::new(UserContext::new(db_context.clone()));
    let access_context = Arc::new(AccessContext::new(db_context.clone()));
    let query_context = Arc::new(QueryContext::new(db_context.clone()));
    let session_context = Arc::new(SessionContext::new(db_context.clone()));
    let in_use_context = Arc::new(InUseContext::new(db_context.clone()));

    ConcreteEcdarApi::new(
        model_context,
        user_context,
        access_context,
        query_context,
        session_context,
        in_use_context,
        mock_ecdar_backend,
    )
    .await
}
