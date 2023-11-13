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
use std::fmt::Debug;
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

    let model_context = Arc::new(ModelContext::new(db_context.clone()));
    let user_context = Arc::new(UserContext::new(db_context.clone()));
    let access_context = Arc::new(AccessContext::new(db_context.clone()));
    let query_context = Arc::new(QueryContext::new(db_context.clone()));
    let session_context = Arc::new(SessionContext::new(db_context.clone()));
    let in_use_context = Arc::new(InUseContext::new(db_context.clone()));

    // Sharing entity contexts works
    print_all_entities(model_context.clone()).await;
    print_all_entities(model_context.clone()).await;
    print_all_entities(model_context.clone()).await;

    print_all_entities(user_context.clone()).await;
    print_all_entities(access_context.clone()).await;
    print_all_entities(query_context.clone()).await;
    print_all_entities(session_context.clone()).await;
    print_all_entities(in_use_context.clone()).await;

    Ok(())
}

async fn print_all_entities<T: Debug>(entity_context: Arc<dyn EntityContextTrait<T>>) {
    let res = entity_context.get_all().await;
    println!("{:?}", res.unwrap())
}
