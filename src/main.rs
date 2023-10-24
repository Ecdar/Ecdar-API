mod api_server;
mod database;
mod entities;

use api_server::server::start_grpc_server;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    start_grpc_server().await.unwrap();
}
