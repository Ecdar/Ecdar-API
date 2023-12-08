//! # Description
//! This project serves as an API server between an ECDAR frontend and Reveaal
//!
//! The project is currently being developed at [Github](https://github.com/ECDAR-AAU-SW-P5/)
//! Ecdar-API serves as the intermediary between the [Ecdar frontend](https://github.com/ECDAR-AAU-SW-P5/Ecdar-GUI-Web) and the [Ecdar backend](https://github.com/ECDAR-AAU-SW-P5/Reveaal) (Reveaal). Its core functionality revolves around storing and managing entities such as users and projects, allowing the backend to focus solely on computations.
//!
//! # Notes
//! Currently, the only supported databases are `PostgreSQL` and `SQLite`
mod api;
mod contexts;
mod controllers;
mod entities;
mod services;
mod tests;

use crate::contexts::context_collection::ContextCollection;
use crate::contexts::context_impls::*;
use crate::contexts::context_traits::DatabaseContextTrait;
use crate::controllers::controller_collection::ControllerCollection;
use crate::controllers::controller_impls::*;
use crate::services::service_collection::ServiceCollection;
use crate::services::service_impls::{HashingService, ReveaalService};
use api::server::start_grpc_server;
use dotenv::dotenv;
use sea_orm::{ConnectionTrait, Database, DbBackend};
use std::env;
use std::error::Error;
use std::sync::Arc;

#[tokio::main]
#[allow(clippy::expect_used)]
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
        reveaal_controller: Arc::new(ReveaalController::new(services.clone())),
    };

    start_grpc_server(logics)
        .await
        .expect("failed to start grpc server");

    Ok(())
}
