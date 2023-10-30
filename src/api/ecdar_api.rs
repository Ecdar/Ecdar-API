use crate::api::server::server::ecdar_api_auth_server::EcdarApiAuth;
use crate::api::server::server::ecdar_api_server::EcdarApi;
use crate::api::server::server::ecdar_backend_client::EcdarBackendClient;
use crate::api::server::server::{CreateUserRequest, CreateUserResponse};
use crate::database::access_context::AccessContext;
use crate::database::database_context::DatabaseContext;
use crate::database::entity_context::EntityContextTrait;
use crate::database::in_use_context::InUseContext;
use crate::database::model_context::ModelContext;
use crate::database::query_context::QueryContext;
use crate::database::session_context::SessionContext;
use crate::database::user_context::UserContext;
use crate::entities::*;
use std::env;
use tonic::{Code, Request, Response, Status};

use super::{
    auth,
    server::server::{
        ecdar_backend_server::EcdarBackend, GetAuthTokenRequest, GetAuthTokenResponse,
        QueryRequest, QueryResponse, SimulationStartRequest, SimulationStepRequest,
        SimulationStepResponse, UserTokenResponse,
    },
};

#[derive(Debug)]
pub struct ConcreteEcdarApi {
    reveaal_address: String,
    db_context: Box<DatabaseContext>,
    model_context: Box<ModelContext>,
    user_context: Box<UserContext>,
    access_context: Box<AccessContext>,
    query_context: Box<QueryContext>,
    session_context: Box<SessionContext>,
    in_use_context: Box<InUseContext>,
}

impl ConcreteEcdarApi {
    pub async fn new(db_context: Box<DatabaseContext>) -> Self {
        ConcreteEcdarApi {
            reveaal_address: env::var("REVEAAL_ADDRESS")
                .expect("Expected REVEAAL_ADDRESS to be set."),
            db_context: db_context.clone(),
            model_context: Box::new(ModelContext::new(db_context.clone())),
            user_context: Box::new(UserContext::new(db_context.clone())),
            access_context: Box::new(AccessContext::new(db_context.clone())),
            query_context: Box::new(QueryContext::new(db_context.clone())),
            session_context: Box::new(SessionContext::new(db_context.clone())),
            in_use_context: Box::new(InUseContext::new(db_context.clone())),
        }
    }
}

#[tonic::async_trait]
impl EcdarApi for ConcreteEcdarApi {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let message = request.into_inner().clone();

        let mut user = user::Model {
            id: Default::default(),
            username: message.clone().username,
            password: message.clone().password,
            email: message.clone().email,
        };

        user = self
            .user_context
            .create(user.clone())
            .await
            .expect("Failed to create user");

        let token = auth::create_jwt(&user.id.to_string()).expect("Failed to create token");

        Ok(Response::new(CreateUserResponse { token }))
    }
}

#[tonic::async_trait]
impl EcdarApiAuth for ConcreteEcdarApi {
    async fn get_auth_token(
        &self,
        request: Request<GetAuthTokenRequest>,
    ) -> Result<Response<GetAuthTokenResponse>, Status> {
        let uid = "1234";
        let token = auth::create_jwt(&uid);

        match token {
            Ok(token) => Ok(Response::new(GetAuthTokenResponse { token })),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }
}

/// Implementation of the EcdarBackend trait, which is used to ensure backwards compatability with the Reveaal engine.
#[tonic::async_trait]
impl EcdarBackend for ConcreteEcdarApi {
    async fn get_user_token(
        &self,
        _request: Request<()>,
    ) -> Result<Response<UserTokenResponse>, Status> {
        let mut client = EcdarBackendClient::connect(self.reveaal_address.clone())
            .await
            .unwrap();
        client.get_user_token(_request).await
    }

    async fn send_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        let mut client = EcdarBackendClient::connect(self.reveaal_address.clone())
            .await
            .unwrap();
        client.send_query(request).await
    }

    async fn start_simulation(
        &self,
        request: Request<SimulationStartRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        let mut client = EcdarBackendClient::connect(self.reveaal_address.clone())
            .await
            .unwrap();
        client.start_simulation(request).await
    }

    async fn take_simulation_step(
        &self,
        request: Request<SimulationStepRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        let mut client = EcdarBackendClient::connect(self.reveaal_address.clone())
            .await
            .unwrap();
        client.take_simulation_step(request).await
    }
}
