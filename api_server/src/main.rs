use core::panic;

use futures::executor::block_on;
use database::{DatabaseContext, EcdarDatabase};

use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_context=    match  DatabaseContext::new().await {
        Ok(db) => db,
        Err(e) => panic!("{:?}", e)
    };

}