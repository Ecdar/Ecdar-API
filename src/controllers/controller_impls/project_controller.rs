use crate::api::auth::{RequestExt, TokenType};
use crate::api::server::protobuf::{
    CreateProjectRequest, CreateProjectResponse, DeleteProjectRequest, GetProjectRequest,
    GetProjectResponse, ListProjectsInfoResponse, Project, Query, UpdateProjectRequest,
};
use crate::contexts::context_collection::ContextCollection;
use crate::controllers::controller_traits::ProjectControllerTrait;
use crate::entities::{access, in_use, project};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use sea_orm::SqlErr;
use tonic::{Code, Request, Response, Status};

const IN_USE_DURATION_MINUTES: i64 = 10;

pub struct ProjectController {
    contexts: ContextCollection,
}

impl ProjectController {
    pub fn new(contexts: ContextCollection) -> Self {
        ProjectController { contexts }
    }
}

#[async_trait]
impl ProjectControllerTrait for ProjectController {
    /// Gets a Model and its queries from the contexts.
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
            .map_err(|_err| Status::internal("could not stringify user id in request metadata"))?
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
            components_info: serde_json::from_value(project.components_info).map_err(|err| {
                Status::internal(format!(
                    "failed to parse components info object, internal error: {}",
                    err
                ))
            })?,
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
                            .get_by_token(
                                TokenType::AccessToken,
                                request
                                    .token_string()
                                    .map_err(|_err| {
                                        Status::internal(
                                            "failed to get token from request metadata",
                                        )
                                    })?
                                    .ok_or(Status::internal(
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
            .map(|query| {
                let result = serde_json::from_value(query.result.unwrap_or_else(|| "".into()))?;

                Ok(Query {
                    id: query.id,
                    project_id: query.project_id,
                    query: query.string,
                    result,
                    outdated: query.outdated,
                })
            })
            .collect::<Result<Vec<Query>, serde_json::Error>>()
            .map_err(|err| {
                Status::internal(format!(
                    "failed to parse json result, inner error:  {}",
                    err
                ))
            })?;

        Ok(Response::new(GetProjectResponse {
            project: Some(project),
            queries,
            in_use: in_use_bool,
        }))
    }

    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
        let message = request.get_ref().clone();
        let uid = request
            .uid()
            .map_err(|_err| Status::internal("could not stringify user id in request metadata"))?
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        let components_info = match message.clone().components_info {
            Some(components_info) => serde_json::to_value(components_info).map_err(|err| {
                Status::internal(format!(
                    "failed to parse components info object, internal error: {}",
                    err
                ))
            })?,
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
            .get_by_token(
                TokenType::AccessToken,
                request
                    .token_string()
                    .map_err(|_err| Status::internal("failed to get token from request metadata"))?
                    .ok_or(Status::internal(
                        "failed to get token from request metadata",
                    ))?,
            )
            .await
            .map_err(|_err| Status::internal("failed to query database"))?
            .ok_or(Status::not_found("token not found"))?;

        let in_use = in_use::Model {
            project_id: project.clone().id,
            session_id: session.id,
            latest_activity: Default::default(),
        };

        self.contexts
            .in_use_context
            .create(in_use)
            .await
            .map_err(|err| {
                Status::internal(format!("a database error occured, internal error: {}", err))
            })?;
        self.contexts
            .access_context
            .create(access)
            .await
            .map_err(|err| {
                Status::internal(format!("a database error occured, internal error: {}", err))
            })?;

        Ok(Response::new(CreateProjectResponse { id: project.id }))
    }

    /// Updates a Model in the contexts given its id.
    ///
    /// # Errors
    /// This function will return an error if the project does not exist in the contexts
    /// or if the user does not have access to the project with role 'Editor'.
    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();
        let uid = request
            .uid()
            .map_err(|_err| Status::internal("could not stringify user id in request metadata"))?
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
            .get_by_token(
                TokenType::AccessToken,
                request
                    .token_string()
                    .map_err(|_err| Status::internal("failed to get token from request metadata"))?
                    .ok_or(Status::internal(
                        "failed to get token from request metadata",
                    ))?,
            )
            .await
        {
            Ok(Some(session)) => session,
            Ok(None) => {
                return Err(Status::unauthenticated(
                    "No session found with given access token",
                ));
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
                Some(components_info) => serde_json::to_value(components_info).map_err(|err| {
                    Status::internal(format!(
                        "failed to parse components info object, internal error: {}",
                        err
                    ))
                })?,
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

    /// Deletes a Model from the contexts.
    ///
    /// # Errors
    /// This function will return an error if the project does not exist in the contexts
    /// or if the user is not the project owner.
    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<()>, Status> {
        let uid = request
            .uid()
            .map_err(|_err| Status::internal("could not stringify user id in request metadata"))?
            .ok_or(Status::internal("Could not get uid from request metadata"))?;
        let project_id = request.get_ref().id;

        let project = match self.contexts.project_context.get_by_id(project_id).await {
            Ok(Some(project)) => project,
            Ok(None) => {
                return Err(Status::new(
                    Code::NotFound,
                    "No project found with given id",
                ));
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
            .map_err(|_err| Status::internal("could not stringify user id in request metadata"))?
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
}

#[cfg(test)]
#[path = "../../tests/controllers/project_controller.rs"]
mod project_controller_tests;
