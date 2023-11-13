use migration::{Migrator, MigratorTrait};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr};
use std::env;
use std::sync::Arc;

#[derive(Clone)]
pub struct PostgresDatabaseContext {
    pub(crate) db_connection: DatabaseConnection,
}

#[derive(Clone)]
pub struct SQLiteDatabaseContext {
    pub(crate) db_connection: DatabaseConnection,
}

#[async_trait]
pub trait DatabaseContextTrait: Send + Sync {
    async fn new(connection_string: &str) -> Result<Self, DbErr>
    where
        Self: Sized;
    async fn reset(&self) -> Result<Arc<dyn DatabaseContextTrait>, DbErr>;
    fn get_connection(&self) -> DatabaseConnection;
    fn get_url(&self) -> String {
        env::var("DATABASE_URL").expect("Expected DATABASE_URL to be set.")
    }
}

#[async_trait]
impl DatabaseContextTrait for PostgresDatabaseContext {
    async fn new(connection_string: &str) -> Result<PostgresDatabaseContext, DbErr> {
        let db = Database::connect(connection_string).await?;

        let db = match db.get_database_backend() {
            DbBackend::Postgres => db,
            _ => panic!("Expected postgresql connection string"),
        };

        Ok(PostgresDatabaseContext { db_connection: db })
    }

    async fn reset(&self) -> Result<Arc<dyn DatabaseContextTrait>, DbErr> {
        Migrator::fresh(&self.db_connection).await.unwrap();

        Ok(Arc::new(PostgresDatabaseContext {
            db_connection: self.get_connection(),
        }))
    }

    fn get_connection(&self) -> DatabaseConnection {
        self.db_connection.clone()
    }
}

#[async_trait]
impl DatabaseContextTrait for SQLiteDatabaseContext {
    async fn new(connection_string: &str) -> Result<SQLiteDatabaseContext, DbErr> {
        let db = Database::connect(connection_string).await?;

        let db = match db.get_database_backend() {
            DbBackend::Sqlite => db,
            _ => panic!("Expected sqlite connection string"),
        };

        Ok(SQLiteDatabaseContext { db_connection: db })
    }

    async fn reset(&self) -> Result<Arc<dyn DatabaseContextTrait>, DbErr> {
        Migrator::fresh(&self.db_connection).await.unwrap();

        Ok(Arc::new(SQLiteDatabaseContext {
            db_connection: self.get_connection(),
        }))
    }

    fn get_connection(&self) -> DatabaseConnection {
        self.db_connection.clone()
    }
}
