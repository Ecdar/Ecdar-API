#[cfg(test)]
mod setup_tests {
    use crate::tests::database::helpers::get_database_backend;
    use dotenv::dotenv;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use std::env;
    #[tokio::main]
    #[ctor::ctor] //DO NOT WRITE TO STDOUT, IT WILL PANIC
    async fn init() {
        dotenv().ok();

        let database_backend = get_database_backend();

        match database_backend {
            DatabaseBackend::Postgres => {
                let conn_string = env::var("TEST_DATABASE_URL").unwrap();

                let (server, database) = conn_string.split_at(conn_string.rfind("/").unwrap());
                let database = &database[1..];

                let db = Database::connect(server).await.unwrap();

                db.execute(Statement::from_string(
                    database_backend,
                    format!("DROP DATABASE IF EXISTS \"{}\";", database),
                ))
                .await
                .unwrap();

                db.execute(Statement::from_string(
                    database_backend,
                    format!("CREATE DATABASE \"{}\";", database),
                ))
                .await
                .unwrap();
            }
            DatabaseBackend::Sqlite => {}
            _ => panic!("Database backend not supported"),
        }
    }
}
