use migration::{Migrator, MigratorTrait};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;

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
    async fn new() -> Result<Self, DbErr>
    where
        Self: Sized;
    async fn reset(&self) -> Result<Box<dyn DatabaseContextTrait>, DbErr>;
    fn get_connection(&self) -> DatabaseConnection;
    fn get_url(&self) -> String {
        env::var("DATABASE_URL").expect("Expected DATABASE_URL to be set.")
    }
}

#[async_trait]
impl DatabaseContextTrait for PostgresDatabaseContext {
    async fn new() -> Result<PostgresDatabaseContext, DbErr> {
        let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL to be set.");
        let db = Database::connect(database_url.clone()).await?;
        Ok(PostgresDatabaseContext { db_connection: db })
    }

    async fn reset(&self) -> Result<Box<dyn DatabaseContextTrait>, DbErr> {
        Migrator::fresh(&self.db_connection).await.unwrap();

        Ok(Box::new(PostgresDatabaseContext {
            db_connection: self.get_connection(),
        }))
    }

    fn get_connection(&self) -> DatabaseConnection {
        self.db_connection.clone()
    }
}

#[async_trait]
impl DatabaseContextTrait for SQLiteDatabaseContext {
    async fn new() -> Result<SQLiteDatabaseContext, DbErr> {
        let connection = Database::connect("sqlite::memory:").await.unwrap();

        Migrator::up(&connection, None).await.unwrap();

        Ok(SQLiteDatabaseContext {
            db_connection: connection,
        })
    }

    async fn reset(&self) -> Result<Box<dyn DatabaseContextTrait>, DbErr> {
        Ok(Box::new(SQLiteDatabaseContext::new().await.unwrap()))
    }

    fn get_connection(&self) -> DatabaseConnection {
        self.db_connection.clone()
    }
}
