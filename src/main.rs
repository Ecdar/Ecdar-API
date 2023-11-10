mod api;
mod database;
mod entities;

use crate::database::access_context::AccessContext;
use crate::database::in_use_context::InUseContext;
use crate::database::model_context::ModelContext;
use crate::database::query_context::QueryContext;
use crate::database::session_context::SessionContext;
use crate::database::user_context::UserContext;
use api::server::start_grpc_server;
use database::database_context::{DatabaseContext, DatabaseContextTrait};
use database::entity_context::EntityContextTrait;
use dotenv::dotenv;
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let db_context = Box::new(DatabaseContext::new().await?);
    let model_context = Arc::new(ModelContext::new(db_context.clone()));
    let user_context = Arc::new(UserContext::new(db_context.clone()));
    let access_context = Arc::new(AccessContext::new(db_context.clone()));
    let query_context = Arc::new(QueryContext::new(db_context.clone()));
    let session_context = Arc::new(SessionContext::new(db_context.clone()));
    let in_use_context = Arc::new(InUseContext::new(db_context.clone()));

    print_all_entities(model_context.clone()).await;
    print_all_entities(user_context.clone()).await;
    print_all_entities(access_context.clone()).await;
    print_all_entities(query_context.clone()).await;
    print_all_entities(session_context.clone()).await;
    print_all_entities(in_use_context.clone()).await;
    start_grpc_server(
        model_context,
        user_context,
        access_context,
        query_context,
        session_context,
        in_use_context,
    )
    .await
    .unwrap();

    Ok(())
}

async fn print_all_entities<T: Debug>(entity_context: Arc<dyn EntityContextTrait<T>>) {
    let res = entity_context.get_all().await;
    println!("{:?}", res.unwrap())
}
