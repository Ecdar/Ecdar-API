#![cfg(test)]

use crate::api::context_collection::ContextCollection;
use crate::api::ecdar_api::ConcreteEcdarApi;
use crate::api::hashing_context::HashingContextTrait;
use crate::api::server::server::ecdar_backend_server::EcdarBackend;
use crate::api::server::server::{
    QueryRequest, QueryResponse, SimulationStartRequest, SimulationStepRequest,
    SimulationStepResponse, UserTokenResponse,
};
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
use tonic::{Request, Response, Status};

pub fn get_mock_concrete_ecdar_api(mock_services: MockServices) -> ConcreteEcdarApi {
    let contexts = ContextCollection {
        access_context: Arc::new(mock_services.access_context_mock),
        in_use_context: Arc::new(mock_services.in_use_context_mock),
        model_context: Arc::new(mock_services.model_context_mock),
        query_context: Arc::new(mock_services.query_context_mock),
        session_context: Arc::new(mock_services.session_context_mock),
        user_context: Arc::new(mock_services.user_context_mock),
        reveaal_context: Arc::new(mock_services.reveaal_context_mock),
        hashing_context: Arc::new(mock_services.hashing_context_mock),
    };
    ConcreteEcdarApi::new(contexts)
}

pub fn get_mock_services() -> MockServices {
    MockServices {
        access_context_mock: MockAccessContext::new(),
        in_use_context_mock: MockInUseContext::new(),
        model_context_mock: MockModelContext::new(),
        query_context_mock: MockQueryContext::new(),
        session_context_mock: MockSessionContext::new(),
        user_context_mock: MockUserContext::new(),
        reveaal_context_mock: MockReveaalContext::new(),
        hashing_context_mock: MockHashingContext::new(),
    }
}

pub struct MockServices {
    pub(crate) access_context_mock: MockAccessContext,
    pub(crate) in_use_context_mock: MockInUseContext,
    pub(crate) model_context_mock: MockModelContext,
    pub(crate) query_context_mock: MockQueryContext,
    pub(crate) session_context_mock: MockSessionContext,
    pub(crate) user_context_mock: MockUserContext,
    pub(crate) reveaal_context_mock: MockReveaalContext,
    pub(crate) hashing_context_mock: MockHashingContext,
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
        async fn get_access_by_uid_and_model_id(
            &self,
            uid: i32,
            model_id: i32,
        ) -> Result<Option<access::Model>, DbErr> {
            access::Entity::find()
                .filter(
                    Condition::all()
                        .add(access::Column::UserId.eq(uid))
                        .add(access::Column::ModelId.eq(model_id)),
                )
                .one(&self.db_context.get_connection())
                .await
        }
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
    impl ModelContextTrait for ModelContext {}
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
        async fn get_all_by_model_id(&self, model_id: i32) -> Result<Vec<query::Model>, DbErr> {
            query::Entity::find()
                .filter(query::Column::ModelId.eq(model_id))
                .all(&self.db_context.get_connection())
                .await
        }
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

mock! {
    pub ReveaalContext {}
    #[async_trait]
    impl EcdarBackend for ReveaalContext {
        async fn get_user_token(&self,request: Request<()>) -> Result<Response<UserTokenResponse>, Status>;
        async fn send_query(&self,request: Request<QueryRequest>) -> Result<Response<QueryResponse>, Status>;
        async fn start_simulation(&self, request: Request<SimulationStartRequest>) -> Result<Response<SimulationStepResponse>, Status>;
        async fn take_simulation_step(&self, request: Request<SimulationStepRequest>) -> Result<Response<SimulationStepResponse>, Status>;
    }
}

mock! {
    pub HashingContext {}
    impl HashingContextTrait for HashingContext {
        fn hash_password(&self, password: String) -> String;
        fn verify_password(&self, password: String, hash: &str) -> bool;
    }
}
