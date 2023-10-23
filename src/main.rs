mod api;
mod database;
mod entities;

use database::database_context::{DatabaseContext, DatabaseContextTrait};
use database::entity_context::EntityContextTrait;
use dotenv::dotenv;
use entities::access::Model as Access;
use crate::database::access_context::AccessContext;
use crate::entities::sea_orm_active_enums::Role;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_context = match DatabaseContext::new().await {
        Ok(db) => db,
        Err(e) => panic!("{:?}", e),
    };

    let access_context = AccessContext::new(db_context);

    let access = Access {
        id: Default::default(),
        role: Role::Editor,
        user_id: 1,
        model_id: 1
    };

    let _res = access_context.create(access).await;
}
