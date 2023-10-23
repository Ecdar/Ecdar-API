mod api;
mod database;
mod entities;

use std::error::Error;
use database::database_context::{DatabaseContext, DatabaseContextTrait};
use database::entity_context::EntityContextTrait;
use dotenv::dotenv;
use crate::database::model_context::{ModelContext, ModelContextTrait};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let db_context: &dyn DatabaseContextTrait = &DatabaseContext::new().await?;

    let model_context: &dyn ModelContextTrait = &ModelContext::new(db_context);

    handle_users(model_context).await;

    Ok(())
}
async fn handle_users(model_context: &dyn ModelContextTrait<'_>) {
    let res = model_context.get_all().await;
    println!("{:?}", res.unwrap())
}