use futures::executor::block_on;
use database::{DatabaseContext, EcdarDatabase};

use dotenv::dotenv;

fn main() {
    dotenv().ok();

    let db_context = database::DatabaseContext {};

    if let Err(err) = block_on(db_context.create_and_connect()) {
        panic!("{}", err);
    }
}