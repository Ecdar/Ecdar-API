use crate::entities::{access, in_use, model, query, session, user};
use migration::{Migrator, MigratorTrait};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DbErr, Schema};
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
    async fn reset(&self) -> Result<Self, DbErr>
    where
        Self: Sized;
    fn get_connection(&self) -> DatabaseConnection;
    fn get_url(&self) -> String {
        env::var("DATABASE_URL").expect("Expected DATABASE_URL to be set.")
    }
}

#[async_trait]
impl DatabaseContextTrait for PostgresDatabaseContext {
    async fn new() -> Result<PostgresDatabaseContext, DbErr>
    where
        Self: Sized,
    {
        let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL to be set.");
        let db = Database::connect(database_url.clone()).await?;
        Ok(PostgresDatabaseContext { db_connection: db })
    }

    async fn reset(&self) -> Result<Self, DbErr>
    where
        Self: Sized,
    {
        let connection = Database::connect("sqlite::memory:").await.unwrap();

        Migrator::up(&connection, None).await.unwrap();

        Ok(PostgresDatabaseContext {
            db_connection: connection,
        })
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

    async fn reset(&self) -> Result<Self, DbErr>
    where
        Self: Sized,
    {
        SQLiteDatabaseContext::new().await
    }

    fn get_connection(&self) -> DatabaseConnection {
        self.db_connection.clone()
    }
}
