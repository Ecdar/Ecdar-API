mod api;
mod database;
mod entities;
mod logics;
mod services;
mod tests;

use crate::api::collections::{ContextCollection, LogicCollection, ServiceCollection};
use crate::database::context_impls::*;
use crate::database::context_traits::DatabaseContextTrait;
use crate::logics::logic_impls::*;
use crate::services::service_impls::{HashingService, ReveaalService};
use api::server::start_grpc_server;
use dotenv::dotenv;
use sea_orm::{ConnectionTrait, Database, DbBackend};
use std::env;
use std::error::Error;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let url = env::var("DATABASE_URL").expect("Expected DATABASE_URL to be set.");
    let db = Database::connect(&url).await?;
    let db_context: Arc<dyn DatabaseContextTrait> = match db.get_database_backend() {
        DbBackend::Sqlite => Arc::new(SQLiteDatabaseContext::new(&url).await?),
        DbBackend::Postgres => Arc::new(PostgresDatabaseContext::new(&url).await?),
        _ => panic!("Database protocol not supported"),
    };

    let contexts = ContextCollection {
        access_context: Arc::new(AccessContext::new(db_context.clone())),
        in_use_context: Arc::new(InUseContext::new(db_context.clone())),
        project_context: Arc::new(ProjectContext::new(db_context.clone())),
        query_context: Arc::new(QueryContext::new(db_context.clone())),
        session_context: Arc::new(SessionContext::new(db_context.clone())),
        user_context: Arc::new(UserContext::new(db_context.clone())),
    };

    let services = ServiceCollection {
        hashing_service: Arc::new(HashingService),
        reveaal_service: Arc::new(ReveaalService),
    };

    let logics = LogicCollection {
        access_logic: Arc::new(AccessLogic::new(contexts.clone())),
        project_logic: Arc::new(ProjectLogic::new(contexts.clone())),
        query_logic: Arc::new(QueryLogic::new(contexts.clone(), services.clone())),
        session_logic: Arc::new(SessionLogic::new(contexts.clone(), services.clone())),
        user_logic: Arc::new(UserLogic::new(contexts.clone(), services.clone())),
        reveaal_logic: Arc::new(()),
    };

    start_grpc_server(logics).await.unwrap();

    Ok(())
}
