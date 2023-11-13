use std::env;
use std::fmt::Debug;

use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{Database, DatabaseConnection, DbErr};

#[derive(Clone, Debug)]
pub struct DatabaseContext {
    pub(crate) db_connection: DatabaseConnection,
}

// impl Debug for dyn DatabaseContextTrait {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("DatabaseContextTrait")
//             .field("db_connection", &self.get_connection())
//             .finish()
//     }
// }

#[async_trait]
pub trait DatabaseContextTrait: Send + Sync + Debug {
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
