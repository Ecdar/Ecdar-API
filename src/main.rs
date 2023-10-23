mod api;
mod database;
mod entities;

use database::database_context::{DatabaseContext, DatabaseContextTrait};
use database::entity_context::EntityContextTrait;
use database::user_context::UserContext;
use dotenv::dotenv;
use entities::user::Model as User;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_context = match DatabaseContext::new().await {
        Ok(db) => db,
        Err(e) => panic!("{:?}", e),
    };

    let user_context = UserContext::new(db_context);

    let anders = User {
        id: Default::default(),
        email: "abemand@hotmail.dk".to_owned(),
        username: "anders_anden".to_owned(),
        password: "rask".to_owned(),
    };

    let _res = user_context.create(anders).await;
}
