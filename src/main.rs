mod api;
mod database;
mod entities;
mod tests;

use crate::database::access_context::AccessContext;
use crate::database::database_context::{PostgresDatabaseContext, SQLiteDatabaseContext};
use crate::database::in_use_context::InUseContext;
use crate::database::model_context::ModelContext;
use crate::database::query_context::QueryContext;
use crate::database::session_context::SessionContext;
use crate::database::user_context::UserContext;
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

    let _model_context = Arc::new(ModelContext::new(db_context.clone()));
    let _user_context = Arc::new(UserContext::new(db_context.clone()));
    let _access_context = Arc::new(AccessContext::new(db_context.clone()));
    let _query_context = Arc::new(QueryContext::new(db_context.clone()));
    let _session_context = Arc::new(SessionContext::new(db_context.clone()));
    let _in_use_context = Arc::new(InUseContext::new(db_context.clone()));

    Ok(())
}
