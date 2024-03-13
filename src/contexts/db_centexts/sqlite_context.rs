use crate::contexts::DatabaseContextTrait;
use migration::{Migrator, MigratorTrait};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr};
use std::sync::Arc;

#[derive(Debug)]
pub struct SQLiteDatabaseContext {
    pub(crate) db_connection: DatabaseConnection,
}

impl SQLiteDatabaseContext {
    pub async fn new(connection_string: &str) -> Result<Self, DbErr> {
        let db = Database::connect(connection_string).await?;

        if db.get_database_backend() != DbBackend::Sqlite {
            panic!("Expected sqlite connection string");
        }

        Ok(Self { db_connection: db })
    }
}

#[async_trait]
impl DatabaseContextTrait for SQLiteDatabaseContext {
    async fn reset(&self) -> Result<Arc<dyn DatabaseContextTrait>, DbErr> {
        Migrator::fresh(&self.db_connection).await?;

        Ok(Arc::new(Self {
            db_connection: self.get_connection(),
        }))
    }

    fn get_connection(&self) -> DatabaseConnection {
        self.db_connection.clone()
    }
}
