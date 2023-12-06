mod api;
mod database;
mod entities;
mod tests;

use crate::api::context_collection::ContextCollection;
use crate::api::hashing_context::HashingContext;
use crate::api::reveaal_context::ReveaalContext;
use crate::database::access_context::AccessContext;
use crate::database::database_context::{PostgresDatabaseContext, SQLiteDatabaseContext};
use crate::database::in_use_context::InUseContext;
use crate::database::project_context::ProjectContext;
use crate::database::query_context::QueryContext;
use crate::database::session_context::SessionContext;
use crate::database::user_context::UserContext;
use api::server::start_grpc_server;
use database::database_context::DatabaseContextTrait;
use database::entity_context::EntityContextTrait;
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
        reveaal_context: Arc::new(ReveaalContext),
        hashing_context: Arc::new(HashingContext),
    };

    start_grpc_server(contexts).await.unwrap();

    Ok(())
}
