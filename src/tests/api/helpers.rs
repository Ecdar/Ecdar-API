#![cfg(test)]

use crate::api::ecdar_api::ConcreteEcdarApi;
use crate::api::reveaal_context::MockReveaalContext;
use crate::api::server::server::ecdar_backend_server::EcdarBackend;
use crate::database::access_context::AccessContext;
use crate::database::database_context::{
    DatabaseContextTrait, PostgresDatabaseContext, SQLiteDatabaseContext,
};
use crate::database::entity_context::EntityContextTrait;
use crate::database::in_use_context::InUseContext;
use crate::database::model_context::ModelContext;
use crate::database::query_context::QueryContext;
use crate::database::session_context::SessionContext;
use crate::database::user_context::{MockUserContextTrait, UserContext};
use sea_orm::{ConnectionTrait, Database, DbBackend};
use std::env;
use std::sync::Arc;

pub async fn get_mock_concrete_ecdar_api(
    mock_user_context: Arc<MockUserContextTrait>,
) -> ConcreteEcdarApi {
    let url = env::var("DATABASE_URL").expect("Expected DATABASE_URL to be set.");
    let db = Database::connect(&url).await?;
    let db_context: Arc<dyn DatabaseContextTrait> = match db.get_database_backend() {
        DbBackend::Sqlite => Arc::new(SQLiteDatabaseContext::new(&url).await?),
        DbBackend::Postgres => Arc::new(PostgresDatabaseContext::new(&url).await?),
        _ => panic!("Database protocol not supported"),
    };

    let user_context = mock_user_context.clone();

    let model_context = Arc::new(ModelContext::new(db_context.clone()));
    let access_context = Arc::new(AccessContext::new(db_context.clone()));
    let query_context = Arc::new(QueryContext::new(db_context.clone()));
    let session_context = Arc::new(SessionContext::new(db_context.clone()));
    let in_use_context = Arc::new(InUseContext::new(db_context.clone()));

    let mock_ecdar_backend = Arc::new(MockReveaalContext::new());

    ConcreteEcdarApi::new(
        model_context,
        user_context,
        access_context,
        query_context,
        session_context,
        in_use_context,
        mock_ecdar_backend,
    )
    .await
}
