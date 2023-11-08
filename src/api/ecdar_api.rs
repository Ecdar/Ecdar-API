use std::env;

use tonic::{Code, Request, Response, Status};

use crate::{api::server::server::ecdar_api_auth_server::EcdarApiAuth, database::user_context::UserContextTrait};
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

use super::{
    auth,
    server::server::{
        ecdar_backend_server::EcdarBackend, GetAuthTokenRequest, GetAuthTokenResponse,
        QueryRequest, QueryResponse, SimulationStartRequest, SimulationStepRequest,
        SimulationStepResponse, UserTokenResponse, DeleteUserRequest, CreateUserRequest, UpdateUserRequest, get_auth_token_request::{AuthOption, user_credentials}
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
    async fn list_models_info(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn get_model(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn create_model(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn update_model(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn delete_model(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn update_user(
        &self,
        _request: Request<UpdateUserRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<()>, Status> {
        // Get uid from request metadata
        let uid = match request.metadata().get("uid").unwrap().to_str() {
            Ok(uid) => uid,
            Err(_) => return Err(Status::new(Code::Internal, "Could not get uid from request metadata")),
        };

        match self.user_context.delete(uid.parse().unwrap()).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    async fn create_access(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn update_access(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn delete_access(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }
}

#[tonic::async_trait]
impl EcdarApiAuth for ConcreteEcdarApi {
    async fn get_auth_token(
        &self,
        request: Request<GetAuthTokenRequest>,
    ) -> Result<Response<GetAuthTokenResponse>, Status> {

        let message = request.get_ref().clone();
        let uid: String;
        let mut username = "".to_string();
        let mut email = "".to_string();
        let mut password = "".to_string();
        match message.auth_option {
            Some(auth_option) => match auth_option {
                AuthOption::RefreshToken(refresh_token) => {
                    let refresh_token = refresh_token;
                    println!("Refresh token: {}", refresh_token);
                }
                AuthOption::UserCredentials(user_credentials) => {
                    match user_credentials.user {
                        Some(user) => match user {
                            user_credentials::User::Username(_username) => {
                                username = _username;
                            }
                            user_credentials::User::Email(_email) => {
                                email = _email;
                            }
                        },
                        None => Err(Status::new(Code::Internal, "No user provided"))?,
                    } 
                    password = user_credentials.password;
                }
            },
            None => Err(Status::new(Code::Internal, "No auth option provided"))?,
        }
        println!("Username: {}", username);
        println!("Email: {}", email);
        println!("Password: {}", password);

        uid = match self.user_context.get_user_by_credentials(email, username, password).await {
            Ok(user) => match user {
                Some(user) => user.id.to_string(),
                None => Err(Status::new(Code::Internal, "No user found"))?,
            },
            Err(error) => Err(Status::new(Code::Internal, error.to_string()))?,
        };

        let access_token = match auth::create_access_token(&uid) {
            Ok(token) => token,
            Err(e) => return Err(Status::new(Code::Internal, e.to_string())),
        };
        let refresh_token = match auth::create_refresh_token(&uid) {
            Ok(token) => token,
            Err(e) => return Err(Status::new(Code::Internal, e.to_string())),
        };
        Ok(Response::new(GetAuthTokenResponse {
            access_token,
            refresh_token,
        }))
    }
    async fn create_user(
        &self,
        _request: Request<CreateUserRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
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
