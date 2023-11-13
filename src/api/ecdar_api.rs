use std::env;
use std::sync::Arc;

use crate::api::ecdar_api::helpers::helpers::{setup_db_with_entities, AnyEntity};
use crate::api::server::server::get_auth_token_request::{user_credentials, AuthOption};
use regex::Regex;
use sea_orm::SqlErr;
use tonic::{Code, Request, Response, Status};

use crate::api::server::server::{
    ecdar_api_auth_server::EcdarApiAuth, ecdar_api_server::EcdarApi,
    ecdar_backend_client::EcdarBackendClient,
};
use crate::database::access_context::AccessContextTrait;
use crate::database::database_context::DatabaseContextTrait;
use crate::database::in_use_context::InUseContextTrait;
use crate::database::model_context::ModelContextTrait;
use crate::database::query_context::QueryContextTrait;
use crate::database::session_context::SessionContextTrait;
use crate::database::user_context::UserContextTrait;
use crate::database::{
    access_context::AccessContext, entity_context::EntityContextTrait,
    in_use_context::InUseContext, model_context::ModelContext, query_context::QueryContext,
    session_context::SessionContext, user_context::UserContext,
};
use crate::entities::user::Model as User;

use super::{
    auth,
    server::server::{
        ecdar_backend_server::EcdarBackend, CreateUserRequest, DeleteUserRequest,
        GetAuthTokenRequest, GetAuthTokenResponse, QueryRequest, QueryResponse,
        SimulationStartRequest, SimulationStepRequest, SimulationStepResponse, UpdateUserRequest,
        UserTokenResponse,
    },
};

#[path = "../tests/database/helpers.rs"]
pub mod helpers;

#[derive(Debug, Clone)]
pub struct ConcreteEcdarApi {
    reveaal_address: String,
    model_context: Arc<dyn ModelContextTrait>,
    user_context: Arc<dyn UserContextTrait>,
    access_context: Arc<dyn AccessContextTrait>,
    query_context: Arc<dyn QueryContextTrait>,
    session_context: Arc<dyn SessionContextTrait>,
    in_use_context: Arc<dyn InUseContextTrait>,
}

fn get_uid_from_request<T>(request: &Request<T>) -> Result<i32, Status> {
    let uid = match request.metadata().get("uid").unwrap().to_str() {
        Ok(uid) => uid,
        Err(_) => {
            return Err(Status::new(
                Code::Internal,
                "Could not get uid from request metadata",
            ));
        }
    };

    Ok(uid.parse().unwrap())
}

impl ConcreteEcdarApi {
    pub async fn new(
        model_context: Arc<dyn ModelContextTrait>,
        user_context: Arc<dyn UserContextTrait>,
        access_context: Arc<dyn AccessContextTrait>,
        query_context: Arc<dyn QueryContextTrait>,
        session_context: Arc<dyn SessionContextTrait>,
        in_use_context: Arc<dyn InUseContextTrait>,
    ) -> Self
    where
        Self: Sized,
    {
        ConcreteEcdarApi {
            reveaal_address: env::var("REVEAAL_ADDRESS")
                .expect("Expected REVEAAL_ADDRESS to be set."),
            model_context,
            user_context,
            access_context,
            query_context,
            session_context,
            in_use_context,
        }
    }
    pub async fn setup_in_memory_db(entities: Vec<AnyEntity>) -> Self {
        let db_context = Box::new(setup_db_with_entities(entities).await);
        env::set_var("REVEAAL_ADDRESS", "");
        ConcreteEcdarApi::new(
            Arc::new(ModelContext::new(db_context.clone())),
            Arc::new(UserContext::new(db_context.clone())),
            Arc::new(AccessContext::new(db_context.clone())),
            Arc::new(QueryContext::new(db_context.clone())),
            Arc::new(SessionContext::new(db_context.clone())),
            Arc::new(InUseContext::new(db_context.clone())),
        )
        .await
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

    /// Updates a user record in the database.
    /// # Errors
    /// Returns an error if the database context fails to update the user or
    /// if the uid could not be parsed from the request metadata.
    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();

        // Get uid from request metadata
        let uid = get_uid_from_request(&request)?;

        // Get new values from request message. Empty string means the value will remain unchanged in the database.
        let new_username = match message.username {
            Some(username) => username,
            None => "".to_string(),
        };

        let new_password = match message.password {
            Some(password) => password,
            None => "".to_string(),
        };

        let new_email = match message.email {
            Some(email) => email,
            None => "".to_string(),
        };

        // Record to be inserted in database
        let user = User {
            id: uid,
            username: new_username.clone(),
            password: new_password.clone(),
            email: new_email.clone(),
        };

        // Update user in database
        match self.user_context.update(user).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
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
        let uid = get_uid_from_request(&request)?;

        // Delete user from database
        match self.user_context.delete(uid).await {
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

fn is_valid_email(email: &str) -> bool {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .unwrap()
        .is_match(email)
}

fn is_valid_username(username: &str) -> bool {
    Regex::new(r"^[a-zA-Z0-9_]{3,32}$")
        .unwrap()
        .is_match(username)
}

#[tonic::async_trait]
impl EcdarApiAuth for ConcreteEcdarApi {
    async fn get_auth_token(
        &self,
        request: Request<GetAuthTokenRequest>,
    ) -> Result<Response<GetAuthTokenResponse>, Status> {
        let message = request.get_ref().clone();
        let uid = match message.auth_option {
            Some(auth_option) => match auth_option {
                AuthOption::RefreshToken(_refresh_token) => {
                    get_uid_from_request(&request).unwrap().to_string()
                }
                AuthOption::UserCredentials(user_credentials) => {
                    if let Some(user) = user_credentials.user {
                        match user {
                            user_credentials::User::Username(username) => {
                                match self.user_context.get_by_username(username).await {
                                    Ok(Some(user)) => user.id.to_string(),
                                    Ok(None) => Err(Status::new(Code::Internal, "No user found"))?,
                                    Err(err) => Err(Status::new(Code::Internal, err.to_string()))?,
                                }
                            }
                            user_credentials::User::Email(email) => {
                                match self.user_context.get_by_email(email).await {
                                    Ok(Some(user)) => user.id.to_string(),
                                    Ok(None) => Err(Status::new(Code::Internal, "No user found"))?,
                                    Err(err) => Err(Status::new(Code::Internal, err.to_string()))?,
                                }
                            }
                        }
                    } else {
                        Err(Status::new(Code::Internal, "No user provided"))?
                    }
                }
            },
            None => Err(Status::new(Code::Internal, "No auth option provided"))?,
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
        request: Request<CreateUserRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.into_inner().clone();

        if !is_valid_username(message.clone().username.as_str()) {
            return Err(Status::new(Code::InvalidArgument, "Invalid username"));
        }

        if !is_valid_email(message.clone().email.as_str()) {
            return Err(Status::new(Code::InvalidArgument, "Invalid email"));
        }

        let user = User {
            id: Default::default(),
            username: message.clone().username,
            password: message.clone().password,
            email: message.clone().email,
        };

        match self.user_context.create(user).await {
            Ok(_) => Ok(Response::new(())),
            Err(e) => match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(e)) => {
                    let error_msg = match e.to_lowercase() {
                        _ if e.contains("username") => "A user with that username already exists",
                        _ if e.contains("email") => "A user with that email already exists",
                        _ => "User already exists",
                    };
                    Err(Status::new(Code::AlreadyExists, error_msg))
                }
                _ => Err(Status::new(Code::Internal, "Could not create user")),
            },
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
#[path = "../tests/api/ecdar_api.rs"]
mod tests;