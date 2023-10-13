use core::panic;

use database::{DatabaseContext, EcdarDatabase};
use futures::executor::block_on;

use entities::entities::user::Model as User;
use database::contexts::EntityContext::EntityContextTrait;
use database::contexts::UserContext::UserContext;
use dotenv::dotenv;

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

    user_context.create(anders).await;
}
