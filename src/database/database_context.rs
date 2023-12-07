use migration::{Migrator, MigratorTrait};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug)]
pub struct PostgresDatabaseContext {
    pub(crate) db_connection: DatabaseConnection,
}

#[derive(Debug)]
pub struct SQLiteDatabaseContext {
    pub(crate) db_connection: DatabaseConnection,
}

#[async_trait]
pub trait DatabaseContextTrait: Send + Sync + Debug {
    async fn reset(&self) -> Result<Arc<dyn DatabaseContextTrait>, DbErr>;
    fn get_connection(&self) -> DatabaseConnection;
}

impl PostgresDatabaseContext {
    pub async fn new(connection_string: &str) -> Result<PostgresDatabaseContext, DbErr> {
        let db = Database::connect(connection_string).await?;

        let db = match db.get_database_backend() {
            DbBackend::Postgres => db,
            _ => panic!("Expected postgresql connection string"),
        };

        Ok(PostgresDatabaseContext { db_connection: db })
    }
}

#[async_trait]
impl DatabaseContextTrait for PostgresDatabaseContext {
    async fn reset(&self) -> Result<Arc<dyn DatabaseContextTrait>, DbErr> {
        Migrator::fresh(&self.db_connection)
            .await?;
            // .expect("failed to connect to database");

        Ok(Arc::new(PostgresDatabaseContext {
            db_connection: self.get_connection(),
        }))
    }

    fn get_connection(&self) -> DatabaseConnection {
        self.db_connection.clone()
    }
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
        Migrator::fresh(&self.db_connection)
            .await?;
            // .expect("failed to connect to database");

        Ok(Arc::new(SQLiteDatabaseContext {
            db_connection: self.get_connection(),
        }))
    }

    fn get_connection(&self) -> DatabaseConnection {
        self.db_connection.clone()
    }
}
