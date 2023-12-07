use super::server::protobuf::{
    ecdar_api_auth_server::EcdarApiAuth,
    ecdar_api_server::EcdarApi,
    ecdar_backend_server::EcdarBackend,
    get_auth_token_request::{user_credentials, UserCredentials},
    CreateAccessRequest, CreateModelRequest, CreateModelResponse, CreateQueryRequest,
    CreateUserRequest, DeleteAccessRequest, DeleteModelRequest, DeleteQueryRequest,
    GetAuthTokenRequest, GetAuthTokenResponse, GetModelRequest, GetModelResponse,
    ListModelsInfoResponse, Query, QueryRequest, QueryResponse, SimulationStartRequest,
    SimulationStepRequest, SimulationStepResponse, UpdateAccessRequest, UpdateModelRequest,
    UpdateQueryRequest, UpdateUserRequest, UserTokenResponse,
};
use crate::api::context_collection::ContextCollection;
use crate::api::{
    auth::{RequestExt, Token, TokenType},
    server::protobuf::Model,
};
use crate::database::{session_context::SessionContextTrait, user_context::UserContextTrait};
use crate::entities::{access, in_use, model, query, session, user};
use chrono::{Duration, Utc};
use regex::Regex;
use sea_orm::{DbErr, SqlErr};
use serde_json;
use std::sync::Arc;
use tonic::{Code, Request, Response, Status};

const IN_USE_DURATION_MINUTES: i64 = 10;

#[derive(Clone)]
pub struct ConcreteEcdarApi {
    contexts: ContextCollection,
}
/// Maps a `DbErr` to a `Status`
// fn to_status(db_err: DbErr) -> Status {
//     //TODO Probably a bad idea to return DbErr messages, oh well.
//     match db_err.sql_err() {
//         Some(serr) => match serr {
//             SqlErr::UniqueConstraintViolation(mes) => return Status::new(Code::AlreadyExists, mes),
//             SqlErr::ForeignKeyConstraintViolation(mes) => {
//                 return Status::new(Code::InvalidArgument, mes)
//             }
//             _ => unreachable!(),
//         },
//         None => {}
//     }
//     match db_err {
//         DbErr::ConnectionAcquire(err) => Status::from_error(Box::new(err)),
//         DbErr::TryIntoErr { from, into, source } => todo!(),
//         DbErr::Conn(err) => Status::new(Code::FailedPrecondition, err.to_string()),
//         DbErr::Exec(err) => Status::new(Code::Internal, err.to_string()),
//         DbErr::Query(err) => Status::new(Code::Internal, err.to_string()),
//         DbErr::ConvertFromU64(mes) => todo!(),
//         DbErr::UnpackInsertId => todo!(),
//         DbErr::UpdateGetPrimaryKey => panic!("unknown error"),
//         DbErr::RecordNotFound(mes) => Status::new(Code::NotFound, mes),
//         DbErr::AttrNotSet(mes) => Status::new(Code::Internal, mes),
//         DbErr::Custom(mes) => Status::new(Code::Unknown, mes),
//         DbErr::Type(mes) => Status::new(Code::Internal, mes),
//         DbErr::Json(mes) => Status::new(Code::InvalidArgument, mes),
//         DbErr::Migration(mes) => todo!(),
//         DbErr::RecordNotInserted => todo!(),
//         DbErr::RecordNotUpdated => Status::new(Code::NotFound, "No record updated"),
//     }
//     // todo!()
// }
/// Updates or creates a session in the database for a given user.
///
///
/// # Errors
/// This function will return an error if the database context returns an error
/// or if a session is not found when trying to update an existing one.
pub async fn handle_session(
    session_context: Arc<dyn SessionContextTrait>,
    request: &Request<GetAuthTokenRequest>,
    is_new_session: bool,
    access_token: String,
    refresh_token: String,
    uid: String,
) -> Result<(), Status> {
    if is_new_session {
        session_context
            .create(session::Model {
                id: Default::default(),
                access_token: access_token.clone(),
                refresh_token: refresh_token.clone(),
                updated_at: Default::default(),
                user_id: uid.parse().map_err(|err| {
                    Status::internal(format!("failed to parse user id (uid) ({err})"))
                })?,
            })
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?;
    } else {
        let mut session = match session_context
            .get_by_token(
                TokenType::RefreshToken,
                request.token_string().ok_or(Status::internal(
                    "failed to get token from request metadata",
                ))?,
            )
            .await
        {
            Ok(Some(session)) => session,
            Ok(None) => {
                return Err(Status::new(
                    Code::Unauthenticated,
                    "No session found with given refresh token",
                ));
            }
            Err(err) => return Err(Status::new(Code::Internal, err.to_string())),
        };

        session.access_token = access_token.clone();
        session.refresh_token = refresh_token.clone();

        session_context
            .update(session)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?;
    }
    Ok(())
}
#[allow(clippy::expect_used)]
fn is_valid_email(email: &str) -> bool {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .expect("failed to compile regex")
        .is_match(email)
}
#[allow(clippy::expect_used)]
fn is_valid_username(username: &str) -> bool {
    Regex::new(r"^[a-zA-Z0-9_]{3,32}$")
        .expect("failed to compile regex")
        .is_match(username)
}

impl ConcreteEcdarApi {
    pub fn new(contexts: ContextCollection) -> Self {
        ConcreteEcdarApi { contexts }
    }
}

#[tonic::async_trait]
impl EcdarApi for ConcreteEcdarApi {
    /// Gets a Model and its queries from the database.
    ///
    /// If the Model is not in use, it will now be in use by the requestees session,
    /// given that they are an Editor.
    async fn get_model(
        &self,
        request: Request<GetModelRequest>,
    ) -> Result<Response<GetModelResponse>, Status> {
        let message = request.get_ref().clone();

        let model_id = message.id;

        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        let access = self
            .contexts
            .access_context
            .get_access_by_uid_and_model_id(uid, model_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| {
                Status::new(Code::PermissionDenied, "User does not have access to model")
            })?;

        let model = self
            .contexts
            .model_context
            .get_by_id(model_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::Internal, "Model not found"))?;

        let model = Model {
            id: model.id,
            name: model.name,
            components_info: serde_json::from_value(model.components_info)
                .map_err(|err| Status::internal(err.to_string()))?,
            owner_id: model.owner_id,
        };

        let mut in_use_bool = true;
        match self.contexts.in_use_context.get_by_id(model_id).await {
            Ok(Some(in_use)) => {
                // If model is not in use and user is an Editor, update the in use with the users session.
                if in_use.latest_activity
                    <= (Utc::now().naive_utc() - Duration::minutes(IN_USE_DURATION_MINUTES))
                {
                    in_use_bool = false;

                    if access.role == "Editor" {
                        let session = self
                            .contexts
                            .session_context
                            .get_by_token(
                                TokenType::AccessToken,
                                request.token_string().ok_or(Status::internal(
                                    "failed to get token from request metadata",
                                ))?,
                            )
                            .await
                            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
                            .ok_or_else(|| {
                                Status::new(
                                    Code::Unauthenticated,
                                    "No session found with given access token",
                                )
                            })?;

                        let in_use = in_use::Model {
                            model_id: in_use.model_id,
                            session_id: session.id,
                            latest_activity: Utc::now().naive_utc(),
                        };

                        self.contexts
                            .in_use_context
                            .update(in_use)
                            .await
                            .map_err(|err| Status::new(Code::Internal, err.to_string()))?;
                    }
                }
            }
            Ok(None) => return Err(Status::new(Code::Internal, "No in use found for model")),
            Err(err) => return Err(Status::new(Code::Internal, err.to_string())),
        }

        let queries = self
            .contexts
            .query_context
            .get_all_by_model_id(model_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?;

        let queries = queries
            .into_iter()
            .map(|query| Query {
                id: query.id,
                model_id: query.model_id,
                query: query.string,
                result: match query.result {
                    Some(result) => {
                        serde_json::from_value(result).expect("failed to parse message")
                    } //TODO better error handling
                    None => "".to_owned(),
                },
                outdated: query.outdated,
            })
            .collect::<Vec<Query>>();

        Ok(Response::new(GetModelResponse {
            model: Some(model),
            queries,
            in_use: in_use_bool,
        }))
    }

    async fn create_model(
        &self,
        request: Request<CreateModelRequest>,
    ) -> Result<Response<CreateModelResponse>, Status> {
        let message = request.get_ref().clone();
        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        let components_info = match message.clone().components_info {
            Some(components_info) => serde_json::to_value(components_info)
                .map_err(|err| Status::internal(err.to_string()))?,
            None => return Err(Status::invalid_argument("No components info provided")),
        };

        let mut model = model::Model {
            id: Default::default(),
            name: message.clone().name,
            components_info,
            owner_id: uid,
        };

        model = match self.contexts.model_context.create(model).await {
            Ok(model) => model,
            Err(error) => {
                return match error.sql_err() {
                    Some(SqlErr::UniqueConstraintViolation(e)) => {
                        let error_msg = match e.to_lowercase() {
                            _ if e.contains("name") => "A model with that name already exists",
                            _ => "Model already exists",
                        };
                        println!("{}", e);
                        Err(Status::already_exists(error_msg))
                    }
                    Some(SqlErr::ForeignKeyConstraintViolation(e)) => {
                        let error_msg = match e.to_lowercase() {
                            _ if e.contains("owner_id") => "No user with that id exists",
                            _ => "Could not create model",
                        };
                        println!("{}", e);
                        Err(Status::invalid_argument(error_msg))
                    }
                    _ => Err(Status::internal(error.to_string())),
                };
            }
        };

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(), //todo!("Use role enum")
            model_id: model.clone().id,
            user_id: uid,
        };

        let session = self
            .contexts
            .session_context
            .get_by_token(
                TokenType::AccessToken,
                request.token_string().ok_or(Status::internal(
                    "Failed to get token from request metadata",
                ))?,
            )
            .await
            .map_err(|_err| Status::internal("failed to query database"))? //TODO better error message
            .ok_or(Status::not_found("token not found"))?;

        let in_use = in_use::Model {
            model_id: model.clone().id,
            session_id: session.id,
            latest_activity: Default::default(),
        };

        self.contexts
            .in_use_context
            .create(in_use)
            .await
            .map_err(|_err| Status::new(Code::Internal, "failed to create entity"))?;

        self.contexts
            .access_context
            .create(access)
            .await
            .map_err(|_err| Status::new(Code::Internal, "failed to create entity"))?;

        Ok(Response::new(CreateModelResponse { id: model.id }))
    }

    /// Updates a Model in the database given its id.
    ///
    /// # Errors
    /// This function will return an error if the model does not exist in the database
    /// or if the user does not have access to the model with role 'Editor'.
    async fn update_model(
        &self,
        request: Request<UpdateModelRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();
        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        // Check if the model exists
        let model = match self.contexts.model_context.get_by_id(message.id).await {
            Ok(Some(model)) => model,
            Ok(None) => return Err(Status::not_found("No model found with given id")),
            Err(error) => return Err(Status::internal(error.to_string())),
        };

        // Check if the user has access to the model
        match self
            .contexts
            .access_context
            .get_access_by_uid_and_model_id(uid, model.id)
            .await
        {
            Ok(access) => {
                let mut is_editor = false;
                let access = match access {
                    Some(access) => {
                        is_editor = access.role == "Editor";
                        Some(access)
                    }
                    None => None,
                };

                if !is_editor || access.is_none() {
                    return Err(Status::permission_denied(
                        "You do not have permission to update this model",
                    ));
                }
            }
            Err(error) => return Err(Status::internal(error.to_string())),
        };

        // Get user session
        let session = match self
            .contexts
            .session_context
            .get_by_token(
                TokenType::AccessToken,
                request.token_string().ok_or(Status::new(
                    Code::Internal,
                    "Failed to get token from request metadata",
                ))?,
            ) //? better error message?
            .await
        {
            Ok(Some(session)) => session,
            Ok(None) => {
                return Err(Status::unauthenticated(
                    "No session found with given access token",
                ))
            }
            Err(error) => return Err(Status::internal(error.to_string())),
        };

        // Get in_use for model
        match self.contexts.in_use_context.get_by_id(model.id).await {
            Ok(Some(in_use)) => {
                // Check if in_use latest activity is older than the max allowed
                if in_use.latest_activity
                    > (Utc::now().naive_utc() - Duration::minutes(IN_USE_DURATION_MINUTES))
                    && in_use.session_id != session.id
                {
                    return Err(Status::failed_precondition(
                        "Model is currently in use by another session",
                    ));
                }

                let new_in_use = in_use::Model {
                    model_id: in_use.model_id,
                    session_id: session.id,
                    latest_activity: Utc::now().naive_utc(),
                };

                match self.contexts.in_use_context.update(new_in_use).await {
                    Ok(_) => (),
                    Err(error) => return Err(Status::internal(error.to_string())),
                }
            }
            Ok(None) => return Err(Status::internal("No in_use found for model")),
            Err(error) => return Err(Status::internal(error.to_string())),
        };

        let new_model = model::Model {
            id: model.id,
            name: match message.clone().name {
                Some(name) => name,
                None => model.name,
            },
            components_info: match message.clone().components_info {
                Some(components_info) => serde_json::to_value(components_info)
                    .map_err(|err| Status::new(Code::Internal, err.to_string()))?,
                None => model.components_info,
            },
            owner_id: match message.clone().owner_id {
                Some(new_owner_id) => {
                    if model.owner_id == uid {
                        new_owner_id
                    } else {
                        return Err(Status::permission_denied(
                            "You do not have permission to change the owner of this model",
                        ));
                    }
                }
                None => model.owner_id,
            },
        };

        match self.contexts.model_context.update(new_model).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    /// Deletes a Model from the database.
    ///
    /// # Errors
    /// This function will return an error if the model does not exist in the database
    /// or if the user is not the model owner.
    async fn delete_model(
        &self,
        request: Request<DeleteModelRequest>,
    ) -> Result<Response<()>, Status> {
        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;
        let model_id = request.get_ref().id;

        let model = self
            .contexts
            .model_context
            .get_by_id(model_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::NotFound, "No model found with the given id"))?;

        // Check if user is owner and thereby has permission to delete model
        if model.owner_id != uid {
            return Err(Status::new(
                Code::PermissionDenied,
                "You do not have permission to delete this model",
            ));
        }

        match self.contexts.model_context.delete(model_id).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => match error {
                sea_orm::DbErr::RecordNotFound(message) => {
                    Err(Status::new(Code::NotFound, message))
                }
                _ => Err(Status::new(Code::Internal, error.to_string())),
            },
        }
    }

    async fn list_models_info(
        &self,
        request: Request<()>,
    ) -> Result<Response<ListModelsInfoResponse>, Status> {
        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        match self
            .contexts
            .model_context
            .get_models_info_by_uid(uid)
            .await
        {
            Ok(model_info_list) => {
                if model_info_list.is_empty() {
                    return Err(Status::new(
                        Code::NotFound,
                        "No access found for given user",
                    ));
                } else {
                    Ok(Response::new(ListModelsInfoResponse { model_info_list }))
                }
            }
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    /// Creates an access in the database.
    /// # Errors
    /// Returns an error if the database context fails to create the access
    async fn create_access(
        &self,
        request: Request<CreateAccessRequest>,
    ) -> Result<Response<()>, Status> {
        let access = request.get_ref();

        let access = access::Model {
            id: Default::default(),
            role: access.role.to_string(),
            model_id: access.model_id,
            user_id: access.user_id,
        };

        match self.contexts.access_context.create(access).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    /// Endpoint for updating an access record.
    ///
    /// Takes `UpdateAccessRequest` as input
    ///
    /// Returns a `Status` as response
    ///
    /// `model_id` and `user_id` is set to 'default' since they won't be updated in the database.
    async fn update_access(
        &self,
        request: Request<UpdateAccessRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();

        let access = access::Model {
            id: message.id,
            role: message.role,
            model_id: Default::default(),
            user_id: Default::default(),
        };

        match self.contexts.access_context.update(access).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    /// Deletes the an Access from the database. This has no sideeffects.
    ///
    /// # Errors
    /// This function will return an error if the access does not exist in the database.
    async fn delete_access(
        &self,
        request: Request<DeleteAccessRequest>,
    ) -> Result<Response<()>, Status> {
        match self
            .contexts
            .access_context
            .delete(request.get_ref().id)
            .await
        {
            Ok(_) => Ok(Response::new(())),
            Err(error) => match error {
                sea_orm::DbErr::RecordNotFound(message) => {
                    Err(Status::new(Code::NotFound, message))
                }
                _ => Err(Status::new(Code::Internal, error.to_string())),
            },
        }
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

        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        // Get user from database
        let user = self
            .contexts
            .user_context
            .get_by_id(uid)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::Internal, "No user found with given uid"))?;

        // Record to be inserted in database
        let new_user = user::Model {
            id: uid,
            username: match message.clone().username {
                Some(username) => {
                    if is_valid_username(username.as_str()) {
                        username
                    } else {
                        return Err(Status::new(Code::InvalidArgument, "Invalid username"));
                    }
                }
                None => user.username,
            },
            email: match message.clone().email {
                Some(email) => {
                    if is_valid_email(email.as_str()) {
                        email
                    } else {
                        return Err(Status::new(Code::InvalidArgument, "Invalid email"));
                    }
                }
                None => user.email,
            },
            password: match message.clone().password {
                Some(password) => self
                    .contexts
                    .hashing_context
                    .hash_password(password)
                    .map_err(|err| {
                        Status::internal(format!("Error hashing user password, message: {err}"))
                    })?,
                None => user.password,
            },
        };

        // Update user in database
        match self.contexts.user_context.update(new_user).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    /// Deletes a user from the database.
    /// # Errors
    /// Returns an error if the database context fails to delete the user or
    /// if the uid could not be parsed from the request metadata.
    async fn delete_user(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        // Delete user from database
        match self.contexts.user_context.delete(uid).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    /// Creates a query in the database
    /// # Errors
    /// Returns an error if the database context fails to create the query or
    async fn create_query(
        &self,
        request: Request<CreateQueryRequest>,
    ) -> Result<Response<()>, Status> {
        let query_request = request.get_ref();
        let query = query::Model {
            id: Default::default(),
            string: query_request.string.to_string(),
            result: Default::default(),
            outdated: Default::default(),
            model_id: query_request.model_id,
        };

        match self.contexts.query_context.create(query).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    /// Endpoint for updating a query record.
    ///
    /// Takes `UpdateQueryRequest` as input
    ///
    /// Returns a `Status` as response
    async fn update_query(
        &self,
        request: Request<UpdateQueryRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();

        let old_query_res = self
            .contexts
            .query_context
            .get_by_id(message.id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?;

        let old_query = match old_query_res {
            Some(oq) => oq,
            None => return Err(Status::new(Code::NotFound, "Query not found".to_string())),
        };

        let query = query::Model {
            id: message.id,
            model_id: Default::default(),
            string: message.string,
            result: old_query.result,
            outdated: old_query.outdated,
        };

        match self.contexts.query_context.update(query).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    /// Deletes a query record in the database.
    /// # Errors
    /// Returns an error if the provided query_id is not found in the database.
    async fn delete_query(
        &self,
        request: Request<DeleteQueryRequest>,
    ) -> Result<Response<()>, Status> {
        match self
            .contexts
            .query_context
            .delete(request.get_ref().id)
            .await
        {
            Ok(_) => Ok(Response::new(())),
            Err(error) => match error {
                sea_orm::DbErr::RecordNotFound(message) => {
                    Err(Status::new(Code::NotFound, message))
                }
                _ => Err(Status::new(Code::Internal, error.to_string())),
            },
        }
    }
}

async fn get_auth_find_user_helper(
    user_context: Arc<dyn UserContextTrait>,
    user_credentials: UserCredentials,
) -> Result<user::Model, Status> {
    if let Some(user) = user_credentials.user {
        match user {
            user_credentials::User::Username(username) => Ok(user_context
                .get_by_username(username)
                .await
                .map_err(|err| Status::new(Code::Internal, err.to_string()))?
                .ok_or_else(|| Status::new(Code::NotFound, "No user found with given username"))?),

            user_credentials::User::Email(email) => Ok(user_context
                .get_by_email(email)
                .await
                .map_err(|err| Status::new(Code::Internal, err.to_string()))?
                .ok_or_else(|| {
                    Status::new(Code::NotFound, "No user found with the given email")
                })?),
        }
    } else {
        Err(Status::new(Code::InvalidArgument, "No user provided"))
    }
}

#[tonic::async_trait]
impl EcdarApiAuth for ConcreteEcdarApi {
    /// This method is used to get a new access and refresh token for a user.
    ///
    /// # Errors
    /// This function will return an error if the user does not exist in the database,
    /// if the password in the request does not match the user's password,
    /// or if no user is provided in the request.
    async fn get_auth_token(
        &self,
        request: Request<GetAuthTokenRequest>,
    ) -> Result<Response<GetAuthTokenResponse>, Status> {
        let message = request.get_ref().clone();
        let uid: String;
        let user_from_db: user::Model;
        let is_new_session: bool;

        // Get user from credentials
        if let Some(user_credentials) = message.user_credentials {
            let input_password = user_credentials.password.clone();
            user_from_db = get_auth_find_user_helper(
                Arc::clone(&self.contexts.user_context),
                user_credentials,
            )
            .await?;

            // Check if password in request matches users password
            if input_password != user_from_db.password {
                return Err(Status::new(Code::Unauthenticated, "Wrong password"));
            }

            uid = user_from_db.id.to_string();

            // Since the user does not have a refresh_token, a new session has to be made
            is_new_session = true;

            // Get user from refresh_token
        } else {
            let refresh_token = Token::from_str(
                TokenType::RefreshToken,
                request
                    .token_str()
                    .ok_or(Status::unauthenticated("No refresh token provided"))?,
            );
            let token_data = refresh_token.validate()?;
            uid = token_data.claims.sub;

            // Since the user does have a refresh_token, a session already exists
            is_new_session = false;
        }
        // Create new access and refresh token with user id
        let access_token = Token::new(TokenType::AccessToken, &uid)?.to_string();
        let refresh_token = Token::new(TokenType::RefreshToken, &uid)?.to_string();

        // Update or create session in database
        handle_session(
            self.contexts.session_context.clone(),
            &request,
            is_new_session,
            access_token.clone(),
            refresh_token.clone(),
            uid,
        )
        .await?;

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

        let user = user::Model {
            id: Default::default(),
            username: message.clone().username,
            password: message.clone().password,
            email: message.clone().email,
        };

        match self.contexts.user_context.create(user).await {
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
        self.contexts.reveaal_context.get_user_token(_request).await
    }

    async fn send_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        self.contexts.reveaal_context.send_query(request).await
    }

    async fn start_simulation(
        &self,
        request: Request<SimulationStartRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        self.contexts
            .reveaal_context
            .start_simulation(request)
            .await
    }

    async fn take_simulation_step(
        &self,
        request: Request<SimulationStepRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        self.contexts
            .reveaal_context
            .take_simulation_step(request)
            .await
    }
}

#[cfg(test)]
#[path = "../tests/api/query_logic.rs"]
mod query_logic_tests;

#[cfg(test)]
#[path = "../tests/api/access_logic.rs"]
mod access_logic_tests;

#[cfg(test)]
#[path = "../tests/api/model_logic.rs"]
mod model_logic_tests;

#[cfg(test)]
#[path = "../tests/api/user_logic.rs"]
mod user_logic_tests;

#[cfg(test)]
#[path = "../tests/api/session_logic.rs"]
mod session_logic_tests;
