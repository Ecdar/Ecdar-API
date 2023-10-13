use sea_orm::{
    ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DbBackend, DbErr, Statement,
};

use sea_orm::prelude::async_trait::async_trait;
use std::env;

pub mod contexts;

pub struct DatabaseContext {
    db: DatabaseConnection,
}

#[async_trait]
pub trait EcdarDatabase {
    async fn new() -> Result<DatabaseContext, DbErr>;
}

#[async_trait]
impl EcdarDatabase for DatabaseContext {
    async fn new() -> Result<DatabaseContext, DbErr> {
        let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL to be set.");

        let db = Database::connect(database_url.clone()).await?;

        match db.get_database_backend() {
            DbBackend::Postgres => {
                let url = format!("{}", database_url.clone());
                Database::connect(&url).await?
            }
            _ => {
                panic!("Database not implemented")
            }
        };

        Ok(DatabaseContext { db: db })
    }
}
