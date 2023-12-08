use crate::contexts::context_traits::DatabaseContextTrait;
use migration::{Migrator, MigratorTrait};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug)]
pub struct SQLiteDatabaseContext {
    pub(crate) db_connection: DatabaseConnection,
}

impl SQLiteDatabaseContext {
    pub async fn new(connection_string: &str) -> Result<SQLiteDatabaseContext, DbErr> {
        let db = Database::connect(connection_string).await?;

        let db = match db.get_database_backend() {
            DbBackend::Sqlite => db,
            _ => panic!("Expected sqlite connection string"),
        };

        Ok(SQLiteDatabaseContext { db_connection: db })
    }
}

#[async_trait]
impl DatabaseContextTrait for SQLiteDatabaseContext {
    async fn reset(&self) -> Result<Arc<dyn DatabaseContextTrait>, DbErr> {
        Migrator::fresh(&self.db_connection).await?;

        Ok(Arc::new(SQLiteDatabaseContext {
            db_connection: self.get_connection(),
        }))
    }

    fn get_connection(&self) -> DatabaseConnection {
        self.db_connection.clone()
    }
}
