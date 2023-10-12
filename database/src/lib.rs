use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DbBackend, DbErr, Statement, DatabaseConnection};

use std::env;
use sea_orm::prelude::async_trait::async_trait;

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
        let db_name = env::var("DB_NAME").expect("Expected DB_NAME to be set.");

        let db = Database::connect(database_url.clone()).await?;

        match db.get_database_backend() {
            DbBackend::Postgres => {
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("DROP DATABASE IF EXISTS \"{}\";", db_name.clone()),
                ))
                    .await?;
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("CREATE DATABASE \"{}\";", db_name.clone()),
                ))
                    .await?;

                let url = format!("{}/{}", database_url.clone(), db_name.clone());
                Database::connect(&url).await?
            }
            _ => {panic!("Database not implemented")}
        };

        Ok(DatabaseContext {db: db})
    }
}

