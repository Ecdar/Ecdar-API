//! # Description
//! This project serves as an API server between an ECDAR frontend and Reveaal
//!
//! The project is currently being developed at [Github](https://github.com/Ecdar/)
//! Ecdar-API serves as the intermediary between the [Ecdar frontend](https://github.com/Ecdar/Ecdar-GUI-Web) and the [Ecdar backend](https://github.com/Ecdar/Reveaal) (Reveaal). Its core functionality revolves around storing and managing entities such as users and projects, allowing the backend to focus solely on computations.
//!
//! # Notes
//! Currently, the only supported databases are `PostgreSQL` and `SQLite`
mod api;
mod contexts;
mod controllers;
mod entities;
mod services;

use api::server::start_grpc_server;
use dotenv::dotenv;
use sea_orm::{ConnectionTrait, Database, DbBackend};
use std::env;
use std::error::Error;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use crate::contexts::{
        AccessContext, ContextCollection, DatabaseContextTrait, InUseContext,
        PostgresDatabaseContext, ProjectContext, QueryContext, SQLiteDatabaseContext,
        SessionContext, UserContext,
    };
    use crate::controllers::*;
    use crate::services::{HashingService, ReveaalService, ServiceCollection};
    dotenv().ok();

    let reveaal_addr = env::var("REVEAAL_ADDRESS").expect("Expected REVEAAL_ADDRESS to be set.");
    let db_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL to be set.");

    let db = Database::connect(&db_url).await?;
    let db_context: Arc<dyn DatabaseContextTrait> = match db.get_database_backend() {
        DbBackend::Sqlite => Arc::new(SQLiteDatabaseContext::new(&db_url).await?),
        DbBackend::Postgres => Arc::new(PostgresDatabaseContext::new(&db_url).await?),
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
        reveaal_service: Arc::new(ReveaalService::new(&reveaal_addr)),
    };

    let controllers = ControllerCollection {
        access_controller: Arc::new(AccessController::new(contexts.clone())),
        project_controller: Arc::new(ProjectController::new(contexts.clone())),
        query_controller: Arc::new(QueryController::new(contexts.clone(), services.clone())),
        session_controller: Arc::new(SessionController::new(contexts.clone(), services.clone())),
        user_controller: Arc::new(UserController::new(contexts.clone(), services.clone())),
        reveaal_controller: Arc::new(ReveaalController::new(services.clone())),
    };

    start_grpc_server(controllers)
        .await
        .expect("failed to start grpc server");

    Ok(())
}
