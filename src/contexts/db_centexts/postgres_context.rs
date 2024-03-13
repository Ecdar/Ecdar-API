use crate::contexts::DatabaseContextTrait;
use async_trait::async_trait;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr};
use std::sync::Arc;

#[derive(Debug)]
pub struct PostgresDatabaseContext {
    pub(crate) db_connection: DatabaseConnection,
}
impl PostgresDatabaseContext {
    pub async fn new(connection_string: &str) -> Result<Self, DbErr> {
        let db = Database::connect(connection_string).await?;

        if db.get_database_backend() != DbBackend::Postgres {
            panic!("Expected postgresql connection string");
        }

        Ok(Self { db_connection: db })
    }
}

#[async_trait]
impl DatabaseContextTrait for PostgresDatabaseContext {
    async fn reset(&self) -> Result<Arc<dyn DatabaseContextTrait>, DbErr> {
        Migrator::fresh(&self.db_connection).await?;

        Ok(Arc::new(PostgresDatabaseContext {
            db_connection: self.get_connection(),
        }))
    }

    fn get_connection(&self) -> DatabaseConnection {
        self.db_connection.clone()
    }
}