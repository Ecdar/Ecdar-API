#![cfg(test)]

use crate::api::ecdar_api::ConcreteEcdarApi;
use crate::api::reveaal_context::ReveaalContext;
use crate::api::server::server::ModelInfo;
use crate::database::access_context::AccessContextTrait;
use crate::database::entity_context::EntityContextTrait;
use crate::database::in_use_context::InUseContextTrait;
use crate::database::model_context::ModelContextTrait;
use crate::database::query_context::QueryContextTrait;
use crate::database::session_context::SessionContextTrait;
use crate::database::user_context::UserContextTrait;
use crate::entities::{access, in_use, model, query, session, user};
use async_trait::async_trait;
use mockall::mock;
use sea_orm::DbErr;
use std::sync::Arc;

pub fn get_mock_concrete_ecdar_api(mock_services: MockServices) -> ConcreteEcdarApi {
    ConcreteEcdarApi::new(
        Arc::new(mock_services.access_context_mock),
        Arc::new(mock_services.in_use_context_mock),
        Arc::new(mock_services.model_context_mock),
        Arc::new(mock_services.query_context_mock),
        Arc::new(mock_services.session_context_mock),
        Arc::new(mock_services.user_context_mock),
        Arc::new(ReveaalContext),
    )
}

pub fn get_mock_services() -> MockServices {
    MockServices {
        access_context_mock: MockAccessContext::new(),
        in_use_context_mock: MockInUseContext::new(),
        model_context_mock: MockModelContext::new(),
        query_context_mock: MockQueryContext::new(),
        session_context_mock: MockSessionContext::new(),
        user_context_mock: MockUserContext::new(),
    }
}

pub struct MockServices {
    pub(crate) access_context_mock: MockAccessContext,
    pub(crate) in_use_context_mock: MockInUseContext,
    pub(crate) model_context_mock: MockModelContext,
    pub(crate) query_context_mock: MockQueryContext,
    pub(crate) session_context_mock: MockSessionContext,
    pub(crate) user_context_mock: MockUserContext,
}

mock! {
    pub AccessContext {}
    #[async_trait]
    impl EntityContextTrait<access::Model> for AccessContext {
        async fn create(&self, entity: access::Model) -> Result<access::Model, DbErr>;
        async fn get_by_id(&self, entity_id: i32) -> Result<Option<access::Model>, DbErr>;
        async fn get_all(&self) -> Result<Vec<access::Model>, DbErr>;
        async fn update(&self, entity: access::Model) -> Result<access::Model, DbErr>;
        async fn delete(&self, entity_id: i32) -> Result<access::Model, DbErr>;
    }
    #[async_trait]
    impl AccessContextTrait for AccessContext {
        async fn get_access_by_uid(&self, uid: i32) -> Result<Vec<access::Model>, DbErr>;
    }
}

mock! {
    pub InUseContext {}
    #[async_trait]
    impl EntityContextTrait<in_use::Model> for InUseContext {
        async fn create(&self, entity: in_use::Model) -> Result<in_use::Model, DbErr>;
        async fn get_by_id(&self, entity_id: i32) -> Result<Option<in_use::Model>, DbErr>;
        async fn get_all(&self) -> Result<Vec<in_use::Model>, DbErr>;
        async fn update(&self, entity: in_use::Model) -> Result<in_use::Model, DbErr>;
        async fn delete(&self, entity_id: i32) -> Result<in_use::Model, DbErr>;
    }
    #[async_trait]
    impl InUseContextTrait for InUseContext {}
}

mock! {
    pub ModelContext {}
    #[async_trait]
    impl EntityContextTrait<model::Model> for ModelContext {
        async fn create(&self, entity: model::Model) -> Result<model::Model, DbErr>;
        async fn get_by_id(&self, entity_id: i32) -> Result<Option<model::Model>, DbErr>;
        async fn get_all(&self) -> Result<Vec<model::Model>, DbErr>;
        async fn update(&self, entity: model::Model) -> Result<model::Model, DbErr>;
        async fn delete(&self, entity_id: i32) -> Result<model::Model, DbErr>;
    }
    #[async_trait]
    impl ModelContextTrait for ModelContext {
        async fn get_model_info_by_uid(&self, uid: i32) -> Result<Vec<ModelInfo>, DbErr>;
    }
}

mock! {
    pub QueryContext {}
    #[async_trait]
    impl EntityContextTrait<query::Model> for QueryContext {
        async fn create(&self, entity: query::Model) -> Result<query::Model, DbErr>;
        async fn get_by_id(&self, entity_id: i32) -> Result<Option<query::Model>, DbErr>;
        async fn get_all(&self) -> Result<Vec<query::Model>, DbErr>;
        async fn update(&self, entity: query::Model) -> Result<query::Model, DbErr>;
        async fn delete(&self, entity_id: i32) -> Result<query::Model, DbErr>;
    }
    #[async_trait]
    impl QueryContextTrait for QueryContext {}
}

mock! {
    pub SessionContext {}
    #[async_trait]
    impl EntityContextTrait<session::Model> for SessionContext {
        async fn create(&self, entity: session::Model) -> Result<session::Model, DbErr>;
        async fn get_by_id(&self, entity_id: i32) -> Result<Option<session::Model>, DbErr>;
        async fn get_all(&self) -> Result<Vec<session::Model>, DbErr>;
        async fn update(&self, entity: session::Model) -> Result<session::Model, DbErr>;
        async fn delete(&self, entity_id: i32) -> Result<session::Model, DbErr>;
    }
    #[async_trait]
    impl SessionContextTrait for SessionContext {
        async fn get_by_refresh_token(&self, refresh_token: String) -> Result<Option<session::Model>, DbErr>;
    }
}

mock! {
    pub UserContext {}
    #[async_trait]
    impl EntityContextTrait<user::Model> for UserContext {
        async fn create(&self, entity: user::Model) -> Result<user::Model, DbErr>;
        async fn get_by_id(&self, entity_id: i32) -> Result<Option<user::Model>, DbErr>;
        async fn get_all(&self) -> Result<Vec<user::Model>, DbErr>;
        async fn update(&self, entity: user::Model) -> Result<user::Model, DbErr>;
        async fn delete(&self, entity_id: i32) -> Result<user::Model, DbErr>;
    }
    #[async_trait]
    impl UserContextTrait for UserContext {
        async fn get_by_username(&self, username: String) -> Result<Option<user::Model>, DbErr>;
        async fn get_by_email(&self, email: String) -> Result<Option<user::Model>, DbErr>;
    }
}
