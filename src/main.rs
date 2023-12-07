mod api;
mod controllers;
mod database;
mod entities;
mod services;
mod tests;

use crate::controllers::controller_collection::ControllerCollection;
use crate::controllers::controller_impls::*;
use crate::database::context_collection::ContextCollection;
use crate::database::context_impls::*;
use crate::database::context_traits::DatabaseContextTrait;
use crate::services::service_collection::ServiceCollection;
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

    let logics = ControllerCollection {
        access_controller: Arc::new(AccessController::new(contexts.clone())),
        project_controller: Arc::new(ProjectController::new(contexts.clone())),
        query_controller: Arc::new(QueryController::new(contexts.clone(), services.clone())),
        session_controller: Arc::new(SessionController::new(contexts.clone(), services.clone())),
        user_controller: Arc::new(UserController::new(contexts.clone(), services.clone())),
        reveaal_controller: Arc::new(()),
    };

    start_grpc_server(logics).await.unwrap();

    Ok(())
}
