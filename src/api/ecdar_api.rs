use std::env;

use tonic::{Code, Request, Response, Status};

use crate::api::server::server::ecdar_api_auth_server::EcdarApiAuth;
use crate::api::server::server::ecdar_api_server::EcdarApi;
use crate::api::server::server::ecdar_backend_client::EcdarBackendClient;
use crate::database::access_context::AccessContext;
use crate::database::database_context::DatabaseContext;
use crate::database::entity_context::EntityContextTrait;
use crate::database::in_use_context::InUseContext;
use crate::database::model_context::ModelContext;
use crate::database::query_context::QueryContext;
use crate::database::session_context::SessionContext;
use crate::database::user_context::UserContext;

use self::helpers::helpers::{setup_db_with_entities, AnyEntity};

use super::{
    auth,
    server::server::{
        ecdar_backend_server::EcdarBackend, CreateUserRequest, CreateUserResponse,
        DeleteUserRequest, GetAuthTokenRequest, GetAuthTokenResponse, QueryRequest, QueryResponse,
        SimulationStartRequest, SimulationStepRequest, SimulationStepResponse, UserTokenResponse,
    },
};

#[path = "../tests/database/helpers.rs"]
pub mod helpers;

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

    pub async fn setup_in_memory_db() -> Self {
        let db_context = setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await;
        
        env::set_var("REVEAAL_ADDRESS", "");
        ConcreteEcdarApi::new(Box::new(db_context)).await
    }
}


#[tonic::async_trait]
impl EcdarApi for ConcreteEcdarApi {
    async fn list_models_info(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn get_model(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn create_model(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn update_model(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn delete_model(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn create_user(
        &self,
        _request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        todo!()
    }

    async fn update_user(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
    }

    /// Deletes a user from the database.
    /// # Errors
    /// Returns an error if the database context fails to delete the user or
    /// if the uid could not be parsed from the request metadata.
    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<()>, Status> {
        // Get uid from request metadata
        let uid = match request.metadata().get("uid").unwrap().to_str() {
            Ok(uid) => uid,
            Err(_) => {
                return Err(Status::new(
                    Code::Internal,
                    "Could not get uid from request metadata",
                ))
            }
        };

        // Delete user from database
        match self.user_context.delete(uid.parse().unwrap()).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    async fn create_access(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn update_access(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn delete_access(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
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


#[cfg(test)]
mod ecdar_api {
    use std::str::FromStr;

    use tonic::{Request, metadata};

    use crate::{api::server::server::ecdar_api_server::EcdarApi, entities::user::Model};

    #[tokio::test]
    async fn delete_user_nonexisting_user_returns_error() -> () {
        let api = super::ConcreteEcdarApi::setup_in_memory_db().await;

        let mut delete_request = Request::new(super::DeleteUserRequest {
            token: "ben".to_owned(),
        });

        // Insert token into request metadata
        delete_request.metadata_mut().insert(
            "uid",
            metadata::MetadataValue::from_str("1").unwrap(),
        );

        let delete_response = api.delete_user(delete_request).await;

        assert!(delete_response.is_err());
    }

    #[tokio::test]
    async fn delete_user_existing_user_returns_ok() -> () {
        let api = super::ConcreteEcdarApi::setup_in_memory_db().await;

        let user = Model {
            id: 1,
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: "123".to_string(),
        };

        let mut delete_request = Request::new(super::DeleteUserRequest {
            token: "shut ur ass up".to_owned(),
        });

        // Insert token into request metadata
        delete_request.metadata_mut().insert(
            "uid",
            metadata::MetadataValue::from_str("1").unwrap(),
        );

        let delete_response = api.delete_user(delete_request).await;

        assert!(delete_response.is_err());
    }
}