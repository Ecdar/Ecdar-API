#![cfg(test)]

use crate::api::auth::TokenType;
use crate::api::server::server::ecdar_backend_server::EcdarBackend;
use crate::api::server::server::AccessInfo;
use crate::api::server::server::ProjectInfo;
use crate::api::server::server::{
    QueryRequest, QueryResponse, SimulationStartRequest, SimulationStepRequest,
    SimulationStepResponse, UserTokenResponse,
};
use crate::database::context_traits::*;
use std::sync::Arc;

use crate::api::collections::{ContextCollection, ServiceCollection};
use crate::entities::{access, in_use, project, query, session, user};
use crate::services::service_traits::*;
use async_trait::async_trait;
use mockall::mock;
use sea_orm::DbErr;
use tonic::{Request, Response, Status};

pub fn get_mock_contexts() -> MockContexts {
    MockContexts {
        access_context_mock: MockAccessContext::new(),
        in_use_context_mock: MockInUseContext::new(),
        project_context_mock: MockProjectContext::new(),
        query_context_mock: MockQueryContext::new(),
        session_context_mock: MockSessionContext::new(),
        user_context_mock: MockUserContext::new(),
    }
}

pub fn get_mock_services() -> MockServices {
    MockServices {
        hashing_service_mock: MockHashingService::new(),
        reveaal_service_mock: MockReveaalService::new(),
    }
}

pub fn disguise_context_mocks(mock_services: MockContexts) -> ContextCollection {
    ContextCollection {
        access_context: Arc::new(mock_services.access_context_mock),
        in_use_context: Arc::new(mock_services.in_use_context_mock),
        project_context: Arc::new(mock_services.project_context_mock),
        query_context: Arc::new(mock_services.query_context_mock),
        session_context: Arc::new(mock_services.session_context_mock),
        user_context: Arc::new(mock_services.user_context_mock),
    }
}

pub fn disguise_service_mocks(mock_services: MockServices) -> ServiceCollection {
    ServiceCollection {
        hashing_service: Arc::new(mock_services.hashing_service_mock),
        reveaal_service: Arc::new(mock_services.reveaal_service_mock),
    }
}

pub struct MockContexts {
    pub(crate) access_context_mock: MockAccessContext,
    pub(crate) in_use_context_mock: MockInUseContext,
    pub(crate) project_context_mock: MockProjectContext,
    pub(crate) query_context_mock: MockQueryContext,
    pub(crate) session_context_mock: MockSessionContext,
    pub(crate) user_context_mock: MockUserContext,
}

pub struct MockServices {
    pub(crate) hashing_service_mock: MockHashingService,
    pub(crate) reveaal_service_mock: MockReveaalService,
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
        async fn get_access_by_uid_and_project_id(
            &self,
            uid: i32,
            project_id: i32,
        ) -> Result<Option<access::Model>, DbErr>;

        async fn get_access_by_project_id(
            &self,
            project_id: i32,
        ) -> Result<Vec<AccessInfo>, DbErr>;
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
    pub ProjectContext {}
    #[async_trait]
    impl EntityContextTrait<project::Model> for ProjectContext {
        async fn create(&self, entity: project::Model) -> Result<project::Model, DbErr>;
        async fn get_by_id(&self, entity_id: i32) -> Result<Option<project::Model>, DbErr>;
        async fn get_all(&self) -> Result<Vec<project::Model>, DbErr>;
        async fn update(&self, entity: project::Model) -> Result<project::Model, DbErr>;
        async fn delete(&self, entity_id: i32) -> Result<project::Model, DbErr>;
    }
    #[async_trait]
    impl ProjectContextTrait for ProjectContext {
        async fn get_project_info_by_uid(&self, uid: i32) -> Result<Vec<ProjectInfo>, DbErr>;
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
    impl QueryContextTrait for QueryContext {
        async fn get_all_by_project_id(&self, project_id: i32) -> Result<Vec<query::Model>, DbErr>;
    }
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
        async fn get_by_token(&self, token_type: TokenType, token: String) -> Result<Option<session::Model>, DbErr>;
        async fn delete_by_token(&self, token_type: TokenType, token: String) -> Result<session::Model, DbErr>;
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
        async fn get_by_ids(&self, ids: Vec<i32>) -> Result<Vec<user::Model>, DbErr>;
    }
}

mock! {
    pub ReveaalService{}
    #[async_trait]
    impl EcdarBackend for ReveaalService {
        async fn get_user_token(&self,request: Request<()>) -> Result<Response<UserTokenResponse>, Status>;
        async fn send_query(&self,request: Request<QueryRequest>) -> Result<Response<QueryResponse>, Status>;
        async fn start_simulation(&self, request: Request<SimulationStartRequest>) -> Result<Response<SimulationStepResponse>, Status>;
        async fn take_simulation_step(&self, request: Request<SimulationStepRequest>) -> Result<Response<SimulationStepResponse>, Status>;
    }
}

mock! {
    pub HashingService {}
    impl HashingServiceTrait for HashingService {
        fn hash_password(&self, password: String) -> String;
        fn verify_password(&self, password: String, hash: &str) -> bool;
    }
}
