use sea_orm::{Database, DatabaseConnection, DbErr};

use sea_orm::prelude::async_trait::async_trait;
use std::env;

#[derive(Clone)]
pub struct DatabaseContext {
    pub(crate) db_connection: DatabaseConnection,
}

#[async_trait]
pub trait DatabaseContextTrait: Send + Sync {
    async fn new() -> Result<Self, DbErr>
    where
        Self: Sized;
    fn get_connection(&self) -> DatabaseConnection;
}

#[async_trait]
impl DatabaseContextTrait for DatabaseContext {
    async fn new() -> Result<DatabaseContext, DbErr> {
        let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL to be set.");
        let db = Database::connect(database_url.clone()).await?;
        Ok(DatabaseContext { db_connection: db })
    }
    fn get_connection(&self) -> DatabaseConnection {
        self.db_connection.clone()
    }
}
