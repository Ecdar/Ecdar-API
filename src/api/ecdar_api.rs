use super::{
    context_collection::ContextCollection,
    server::server::{
        create_access_request::User,
        ecdar_api_auth_server::EcdarApiAuth,
        ecdar_api_server::EcdarApi,
        ecdar_backend_server::EcdarBackend,
        get_auth_token_request::{user_credentials, UserCredentials},
        CreateAccessRequest, CreateProjectRequest, CreateProjectResponse, CreateQueryRequest,
        CreateUserRequest, DeleteAccessRequest, DeleteProjectRequest, DeleteQueryRequest,
        GetAuthTokenRequest, GetAuthTokenResponse, GetProjectRequest, GetProjectResponse,
        GetUsersRequest, GetUsersResponse, ListAccessInfoRequest, ListAccessInfoResponse,
        ListProjectsInfoResponse, Query, QueryRequest, QueryResponse, SendQueryRequest,
        SendQueryResponse, SimulationStartRequest, SimulationStepRequest, SimulationStepResponse,
        UpdateAccessRequest, UpdateProjectRequest, UpdateQueryRequest, UpdateUserRequest,
        UserTokenResponse,
    },
};
use crate::database::{session_context::SessionContextTrait, user_context::UserContextTrait};
use crate::entities::{access, in_use, project, query, session, user};
use crate::{
    api::{
        auth::{RequestExt, Token, TokenType},
        server::server::Project,
    },
    database::access_context::AccessContextTrait,
};
use chrono::{Duration, Utc};
use regex::Regex;
use sea_orm::SqlErr;
use serde_json;
use std::sync::Arc;
use tonic::{Code, Request, Response, Status};

const IN_USE_DURATION_MINUTES: i64 = 10;

#[derive(Clone)]
pub struct ConcreteEcdarApi {
    contexts: ContextCollection,
}

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
                user_id: uid.parse().unwrap(),
            })
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?;
    } else {
        let mut session = match session_context
            .get_by_token(TokenType::RefreshToken, request.token_string().unwrap())
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

impl ConcreteEcdarApi {
    pub fn new(contexts: ContextCollection) -> Self {
        ConcreteEcdarApi { contexts }
    }
}

#[tonic::async_trait]
impl EcdarApi for ConcreteEcdarApi {
    async fn get_users(
        &self,
        _request: Request<GetUsersRequest>,
    ) -> Result<Response<GetUsersResponse>, Status> {
        todo!()
    }

    async fn delete_session(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
    }

    /// Gets a Model and its queries from the database.
    ///
    /// If the Model is not in use, it will now be in use by the requestees session,
    /// given that they are an Editor.
    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<GetProjectResponse>, Status> {
        let message = request.get_ref().clone();

        let project_id = message.id;

        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        let access = self
            .contexts
            .access_context
            .get_access_by_uid_and_project_id(uid, project_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| {
                Status::new(
                    Code::PermissionDenied,
                    "User does not have access to project",
                )
            })?;

        let project = self
            .contexts
            .project_context
            .get_by_id(project_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::Internal, "Model not found"))?;

        let project = Project {
            id: project.id,
            name: project.name,
            components_info: serde_json::from_value(project.components_info).unwrap(),
            owner_id: project.owner_id,
        };

        let mut in_use_bool = true;
        match self.contexts.in_use_context.get_by_id(project_id).await {
            Ok(Some(in_use)) => {
                // If project is not in use and user is an Editor, update the in use with the users session.
                if in_use.latest_activity
                    <= (Utc::now().naive_utc() - Duration::minutes(IN_USE_DURATION_MINUTES))
                {
                    in_use_bool = false;

                    if access.role == "Editor" {
                        let session = self
                            .contexts
                            .session_context
                            .get_by_token(TokenType::AccessToken, request.token_string().unwrap())
                            .await
                            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
                            .ok_or_else(|| {
                                Status::new(
                                    Code::Unauthenticated,
                                    "No session found with given access token",
                                )
                            })?;

                        let in_use = in_use::Model {
                            project_id: in_use.project_id,
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
            Ok(None) => return Err(Status::new(Code::Internal, "No in use found for project")),
            Err(err) => return Err(Status::new(Code::Internal, err.to_string())),
        }

        let queries = self
            .contexts
            .query_context
            .get_all_by_project_id(project_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?;

        let queries = queries
            .into_iter()
            .map(|query| Query {
                id: query.id,
                project_id: query.project_id,
                query: query.string,
                result: match query.result {
                    Some(result) => serde_json::from_value(result).unwrap(),
                    None => "".to_owned(),
                },
                outdated: query.outdated,
            })
            .collect::<Vec<Query>>();

        Ok(Response::new(GetProjectResponse {
            project: Some(project),
            queries,
            in_use: in_use_bool,
        }))
    }

    async fn list_access_info(
        &self,
        request: Request<ListAccessInfoRequest>,
    ) -> Result<Response<ListAccessInfoResponse>, Status> {
        let message = request.get_ref().clone();

        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        match self
            .contexts
            .access_context
            .get_access_by_uid_and_project_id(uid, message.project_id)
            .await
        {
            Ok(access) => {
                if access.is_none() {
                    return Err(Status::new(
                        Code::PermissionDenied,
                        "User does not have access to model",
                    ));
                }
            }
            Err(error) => return Err(Status::new(Code::Internal, error.to_string())),
        };

        match self
            .contexts
            .access_context
            .get_access_by_project_id(message.project_id)
            .await
        {
            Ok(access_info_list) => {
                if access_info_list.is_empty() {
                    return Err(Status::new(
                        Code::NotFound,
                        "No access found for given user",
                    ));
                } else {
                    Ok(Response::new(ListAccessInfoResponse { access_info_list }))
                }
            }
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
        let message = request.get_ref().clone();
        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        let components_info = match message.clone().components_info {
            Some(components_info) => serde_json::to_value(components_info).unwrap(),
            None => return Err(Status::invalid_argument("No components info provided")),
        };

        let mut project = project::Model {
            id: Default::default(),
            name: message.clone().name,
            components_info,
            owner_id: uid,
        };

        project = match self.contexts.project_context.create(project).await {
            Ok(project) => project,
            Err(error) => {
                return match error.sql_err() {
                    Some(SqlErr::UniqueConstraintViolation(e)) => {
                        let error_msg = match e.to_lowercase() {
                            _ if e.contains("name") => "A project with that name already exists",
                            _ => "Model already exists",
                        };
                        println!("{}", e);
                        Err(Status::already_exists(error_msg))
                    }
                    Some(SqlErr::ForeignKeyConstraintViolation(e)) => {
                        let error_msg = match e.to_lowercase() {
                            _ if e.contains("owner_id") => "No user with that id exists",
                            _ => "Could not create project",
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
            project_id: project.clone().id,
            user_id: uid,
        };

        let session = self
            .contexts
            .session_context
            .get_by_token(TokenType::AccessToken, request.token_string().unwrap())
            .await
            .unwrap()
            .unwrap();

        let in_use = in_use::Model {
            project_id: project.clone().id,
            session_id: session.id,
            latest_activity: Default::default(),
        };

        self.contexts.in_use_context.create(in_use).await.unwrap();
        self.contexts.access_context.create(access).await.unwrap();

        Ok(Response::new(CreateProjectResponse { id: project.id }))
    }

    /// Updates a Model in the database given its id.
    ///
    /// # Errors
    /// This function will return an error if the project does not exist in the database
    /// or if the user does not have access to the project with role 'Editor'.
    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();
        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        // Check if the project exists
        let project = match self.contexts.project_context.get_by_id(message.id).await {
            Ok(Some(project)) => project,
            Ok(None) => return Err(Status::not_found("No project found with given id")),
            Err(error) => return Err(Status::internal(error.to_string())),
        };

        // Check if the user has access to the project
        match self
            .contexts
            .access_context
            .get_access_by_uid_and_project_id(uid, project.id)
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
                        "You do not have permission to update this project",
                    ));
                }
            }
            Err(error) => return Err(Status::internal(error.to_string())),
        };

        // Get user session
        let session = match self
            .contexts
            .session_context
            .get_by_token(TokenType::AccessToken, request.token_string().unwrap())
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

        // Get in_use for project
        match self.contexts.in_use_context.get_by_id(project.id).await {
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
                    project_id: in_use.project_id,
                    session_id: session.id,
                    latest_activity: Utc::now().naive_utc(),
                };

                match self.contexts.in_use_context.update(new_in_use).await {
                    Ok(_) => (),
                    Err(error) => return Err(Status::internal(error.to_string())),
                }
            }
            Ok(None) => return Err(Status::internal("No in_use found for project")),
            Err(error) => return Err(Status::internal(error.to_string())),
        };

        let new_project = project::Model {
            id: project.id,
            name: match message.clone().name {
                Some(name) => name,
                None => project.name,
            },
            components_info: match message.clone().components_info {
                Some(components_info) => serde_json::to_value(components_info).unwrap(),
                None => project.components_info,
            },
            owner_id: match message.clone().owner_id {
                Some(new_owner_id) => {
                    if project.owner_id == uid {
                        new_owner_id
                    } else {
                        return Err(Status::permission_denied(
                            "You do not have permission to change the owner of this project",
                        ));
                    }
                }
                None => project.owner_id,
            },
        };

        match self.contexts.project_context.update(new_project).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    /// Deletes a Model from the database.
    ///
    /// # Errors
    /// This function will return an error if the project does not exist in the database
    /// or if the user is not the project owner.
    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<()>, Status> {
        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;
        let project_id = request.get_ref().id;

        let project = match self.contexts.project_context.get_by_id(project_id).await {
            Ok(Some(project)) => project,
            Ok(None) => {
                return Err(Status::new(
                    Code::NotFound,
                    "No project found with given id",
                ))
            }
            Err(err) => return Err(Status::new(Code::Internal, err.to_string())),
        };

        // Check if user is owner and thereby has permission to delete project
        if project.owner_id != uid {
            return Err(Status::new(
                Code::PermissionDenied,
                "You do not have permission to delete this project",
            ));
        }

        match self.contexts.project_context.delete(project_id).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => match error {
                sea_orm::DbErr::RecordNotFound(message) => {
                    Err(Status::new(Code::NotFound, message))
                }
                _ => Err(Status::new(Code::Internal, error.to_string())),
            },
        }
    }

    async fn list_projects_info(
        &self,
        request: Request<()>,
    ) -> Result<Response<ListProjectsInfoResponse>, Status> {
        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        match self
            .contexts
            .project_context
            .get_project_info_by_uid(uid)
            .await
        {
            Ok(project_info_list) => {
                if project_info_list.is_empty() {
                    return Err(Status::new(
                        Code::NotFound,
                        "No access found for given user",
                    ));
                } else {
                    Ok(Response::new(ListProjectsInfoResponse {
                        project_info_list,
                    }))
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
        let message = request.get_ref().clone();

        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        // Check if the requester has access to model with role 'Editor'
        check_editor_role_helper(
            Arc::clone(&self.contexts.access_context),
            uid,
            message.project_id,
        )
        .await?;

        if let Some(user) = message.user {
            let user_from_db =
                create_access_find_user_helper(Arc::clone(&self.contexts.user_context), user)
                    .await?;

            let access = access::Model {
                id: Default::default(),
                role: message.role.to_string(),
                project_id: message.project_id,
                user_id: user_from_db.id,
            };

            match self.contexts.access_context.create(access).await {
                Ok(_) => Ok(Response::new(())),
                Err(error) => Err(Status::new(Code::Internal, error.to_string())),
            }
        } else {
            Err(Status::new(
                Code::InvalidArgument,
                "No user identification provided",
            ))
        }
    }

    /// Endpoint for updating an access record.
    ///
    /// Takes `UpdateAccessRequest` as input
    ///
    /// Returns a `Status` as response
    ///
    /// `project_id` and `user_id` is set to 'default' since they won't be updated in the database.
    async fn update_access(
        &self,
        request: Request<UpdateAccessRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();

        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        let user_access = self
            .contexts
            .access_context
            .get_by_id(message.id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| {
                Status::new(
                    Code::NotFound,
                    "No access entity found for user".to_string(),
                )
            })?;

        check_editor_role_helper(
            Arc::clone(&self.contexts.access_context),
            uid,
            user_access.project_id,
        )
        .await?;

        let model = self
            .contexts
            .project_context
            .get_by_id(user_access.project_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::NotFound, "No model found for access".to_string()))?;

        // Check that the requester is not trying to update the owner's access
        if model.owner_id == message.id {
            return Err(Status::new(
                Code::PermissionDenied,
                "Requester does not have permission to update access for this user",
            ));
        }

        let access = access::Model {
            id: message.id,
            role: message.role,
            project_id: Default::default(),
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
        let message = request.get_ref().clone();

        let uid = request
            .uid()
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        let user_access = self
            .contexts
            .access_context
            .get_by_id(message.id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| {
                Status::new(
                    Code::NotFound,
                    "No access entity found for user".to_string(),
                )
            })?;

        check_editor_role_helper(
            Arc::clone(&self.contexts.access_context),
            uid,
            user_access.project_id,
        )
        .await?;

        let model = self
            .contexts
            .project_context
            .get_by_id(user_access.project_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::NotFound, "No model found for access".to_string()))?;

        // Check that the requester is not trying to delete the owner's access
        if model.owner_id == message.id {
            return Err(Status::new(
                Code::PermissionDenied,
                "You cannot delete the access entity for this user",
            ));
        }

        match self.contexts.access_context.delete(message.id).await {
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
                Some(password) => self.contexts.hashing_context.hash_password(password),
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

        let access = self
            .contexts
            .access_context
            .get_access_by_uid_and_project_id(request.uid().unwrap(), query_request.project_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| {
                Status::new(
                    Code::PermissionDenied,
                    "User does not have access to project",
                )
            })?;

        if access.role != "Editor" {
            return Err(Status::new(
                Code::PermissionDenied,
                "Role does not have permission to create query",
            ));
        }

        let query = query::Model {
            id: Default::default(),
            string: query_request.string.to_string(),
            result: Default::default(),
            outdated: Default::default(),
            project_id: query_request.project_id,
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

        let access = self
            .contexts
            .access_context
            .get_access_by_uid_and_project_id(request.uid().unwrap(), old_query.project_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| {
                Status::new(
                    Code::PermissionDenied,
                    "User does not have access to project",
                )
            })?;

        if access.role != "Editor" {
            return Err(Status::new(
                Code::PermissionDenied,
                "Role does not have permission to update query",
            ));
        }

        let query = query::Model {
            id: message.id,
            project_id: Default::default(),
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
        let message = request.get_ref();

        let query = self
            .contexts
            .query_context
            .get_by_id(message.id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::NotFound, "Query not found"))?;

        let access = self
            .contexts
            .access_context
            .get_access_by_uid_and_project_id(request.uid().unwrap(), query.project_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| {
                Status::new(
                    Code::PermissionDenied,
                    "User does not have access to project",
                )
            })?;

        if access.role != "Editor" {
            return Err(Status::new(
                Code::PermissionDenied,
                "Role does not have permission to update query",
            ));
        }

        match self.contexts.query_context.delete(message.id).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => match error {
                sea_orm::DbErr::RecordNotFound(message) => {
                    Err(Status::new(Code::NotFound, message))
                }
                _ => Err(Status::new(Code::Internal, error.to_string())),
            },
        }
    }

    /// Sends a query to be run on Reveaal.
    /// After query is run the result is stored in the database.
    ///  
    /// Returns the response that is received from Reveaal.
    async fn send_query(
        &self,
        request: Request<SendQueryRequest>,
    ) -> Result<Response<SendQueryResponse>, Status> {
        let message = request.get_ref();

        let uid = request.uid().unwrap();

        // Verify user access
        self.contexts
            .access_context
            .get_access_by_uid_and_project_id(uid, message.project_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| {
                Status::new(
                    Code::PermissionDenied,
                    "User does not have access to project",
                )
            })?;

        // Get project from database
        let project = self
            .contexts
            .project_context
            .get_by_id(message.project_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::NotFound, "Model not found"))?;

        // Get query from database
        let query = self
            .contexts
            .query_context
            .get_by_id(message.id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::NotFound, "Query not found"))?;

        // Construct query request to send to Reveaal
        let query_request = Request::new(QueryRequest {
            user_id: uid,
            query_id: message.id,
            query: query.string.clone(),
            components_info: serde_json::from_value(project.components_info).unwrap(),
            settings: Default::default(), //TODO
        });

        // Run query on Reveaal
        let query_result = self
            .contexts
            .reveaal_context
            .send_query(query_request)
            .await?;

        // Update query result in database
        self.contexts
            .query_context
            .update(query::Model {
                id: query.id,
                string: query.string.clone(),
                result: Some(
                    serde_json::to_value(query_result.get_ref().result.clone().unwrap()).unwrap(),
                ),
                outdated: false,
                project_id: query.project_id,
            })
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?;

        Ok(Response::new(SendQueryResponse {
            response: Some(query_result.into_inner()),
        }))
    }

    /// Deletes the requester's session, found by their access token.
    ///  
    /// Returns the response that is received from Reveaal.
    async fn delete_session(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let access_token = request
            .token_string()
            .ok_or(Status::unauthenticated("No access token provided"))?;

        match self
            .contexts
            .session_context
            .delete_by_token(TokenType::AccessToken, access_token)
            .await
        {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }
}

async fn check_editor_role_helper(
    access_context: Arc<dyn AccessContextTrait>,
    user_id: i32,
    project_id: i32,
) -> Result<(), Status> {
    let access = access_context
        .get_access_by_uid_and_project_id(user_id, project_id)
        .await
        .map_err(|err| Status::new(Code::Internal, err.to_string()))?
        .ok_or_else(|| {
            Status::new(
                Code::PermissionDenied,
                "User does not have access to model".to_string(),
            )
        })?;

    // Check if the requester has role 'Editor'
    if access.role != "Editor" {
        return Err(Status::new(
            Code::PermissionDenied,
            "User does not have 'Editor' role for this model",
        ));
    }

    Ok(())
}

async fn create_access_find_user_helper(
    user_context: Arc<dyn UserContextTrait>,
    user: User,
) -> Result<user::Model, Status> {
    match user {
        User::UserId(user_id) => Ok(user_context
            .get_by_id(user_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::NotFound, "No user found with given id"))?),

        User::Username(username) => Ok(user_context
            .get_by_username(username)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::NotFound, "No user found with given username"))?),

        User::Email(email) => Ok(user_context
            .get_by_email(email)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::NotFound, "No user found with given email"))?),
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
#[path = "../tests/api/project_logic.rs"]
mod project_logic_tests;

#[cfg(test)]
#[path = "../tests/api/user_logic.rs"]
mod user_logic_tests;

#[cfg(test)]
#[path = "../tests/api/session_logic.rs"]
mod session_logic_tests;
