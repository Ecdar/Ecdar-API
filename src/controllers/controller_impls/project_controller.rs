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
    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<GetProjectResponse>, Status> {
        let message = request.get_ref().clone();

        let project_id = message.id;

        let uid = request
            .uid()
            .map_err(|err| {
                Status::internal(format!(
                    "could not stringify user id in request metadata, internal error {}",
                    err
                ))
            })?
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
                                    .map_err(|err| Status::internal(format!("could not stringify user id in request metadata, internal error {}",err)))?
                                    .ok_or(Status::invalid_argument("failed to get token from request metadata"))?,
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
            .map_err(|err| {
                Status::internal(format!(
                    "could not stringify user id in request metadata, internal error {}",
                    err
                ))
            })?
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
                    .map_err(|err| {
                        Status::internal(format!(
                            "could not stringify user id in request metadata, internal error {}",
                            err
                        ))
                    })?
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

    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();
        let uid = request
            .uid()
            .map_err(|err| {
                Status::internal(format!(
                    "could not stringify user id in request metadata, internal error {}",
                    err
                ))
            })?
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
                    .map_err(|err| {
                        Status::internal(format!(
                            "could not stringify user id in request metadata, internal error {}",
                            err
                        ))
                    })?
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

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<()>, Status> {
        let uid = request
            .uid()
            .map_err(|err| {
                Status::internal(format!(
                    "could not stringify user id in request metadata, internal error {}",
                    err
                ))
            })?
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
            .map_err(|err| {
                Status::internal(format!(
                    "could not stringify user id in request metadata, internal error {}",
                    err
                ))
            })?
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
mod tests {
    use super::super::helpers::disguise_context_mocks;
    use super::super::helpers::get_mock_contexts;
    use crate::controllers::controller_impls::ProjectController;
    use crate::controllers::controller_traits::ProjectControllerTrait;
    use crate::{
        api::{
            auth::TokenType,
            server::protobuf::{
                component::Rep, Component, ComponentsInfo, CreateProjectRequest,
                DeleteProjectRequest, GetProjectRequest, ProjectInfo, UpdateProjectRequest,
            },
        },
        entities::{access, in_use, project, query, session},
    };
    use chrono::Utc;
    use mockall::predicate;
    use sea_orm::DbErr;
    use std::str::FromStr;
    use tonic::{metadata, Code, Request};

    #[tokio::test]
    async fn create_project_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

        let uid = 0;

        let components_info = ComponentsInfo {
            components: vec![],
            components_hash: 0,
        };

        let project = project::Model {
            id: Default::default(),
            name: Default::default(),
            components_info: serde_json::to_value(components_info.clone()).unwrap(),
            owner_id: uid,
        };

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            user_id: uid,
            project_id: project.id,
        };

        let session = session::Model {
            id: Default::default(),
            refresh_token: "refresh_token".to_string(),
            access_token: "access_token".to_string(),
            updated_at: Default::default(),
            user_id: uid,
        };

        let in_use = in_use::Model {
            project_id: project.id,
            session_id: session.id,
            latest_activity: Default::default(),
        };

        mock_contexts
            .project_context_mock
            .expect_create()
            .with(predicate::eq(project.clone()))
            .returning(move |_| Ok(project.clone()));

        mock_contexts
            .access_context_mock
            .expect_create()
            .with(predicate::eq(access.clone()))
            .returning(move |_| Ok(access.clone()));

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .with(
                predicate::eq(TokenType::AccessToken),
                predicate::eq("access_token".to_string()),
            )
            .returning(move |_, _| Ok(Some(session.clone())));

        mock_contexts
            .in_use_context_mock
            .expect_create()
            .with(predicate::eq(in_use.clone()))
            .returning(move |_| Ok(in_use.clone()));

        let mut request = Request::new(CreateProjectRequest {
            name: Default::default(),
            components_info: Option::from(components_info),
        });

        request
            .metadata_mut()
            .insert("uid", uid.to_string().parse().unwrap());

        request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
        );

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.create_project(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn create_project_existing_name_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        let uid = 0;

        let project = project::Model {
            id: Default::default(),
            name: "project".to_string(),
            components_info: Default::default(),
            owner_id: uid,
        };

        mock_contexts
            .project_context_mock
            .expect_create()
            .with(predicate::eq(project.clone()))
            .returning(move |_| Err(DbErr::RecordNotInserted)); //todo!("Needs to be a SqlError with UniqueConstraintViolation with 'name' in message)

        let mut request = Request::new(CreateProjectRequest {
            name: "project".to_string(),
            components_info: Default::default(),
        });

        request
            .metadata_mut()
            .insert("uid", uid.to_string().parse().unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.create_project(request).await;

        assert_eq!(res.unwrap_err().code(), Code::InvalidArgument); //todo!("Needs to be code AlreadyExists when mocked Error is corrected)
    }

    #[tokio::test]
    async fn get_project_user_has_access_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

        let project = project::Model {
            id: Default::default(),
            name: "project".to_string(),
            components_info: Default::default(),
            owner_id: 0,
        };

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: 1,
            user_id: 1,
        };

        let in_use = in_use::Model {
            project_id: Default::default(),
            session_id: 0,
            latest_activity: Utc::now().naive_utc(),
        };

        let queries: Vec<query::Model> = vec![];

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(0), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(project.clone())));

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(in_use.clone())));

        mock_contexts
            .query_context_mock
            .expect_get_all_by_project_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(queries.clone()));

        let mut request = Request::new(GetProjectRequest { id: 0 });

        request.metadata_mut().insert("uid", "0".parse().unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.get_project(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn delete_not_owner_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: 1,
                    name: Default::default(),
                    components_info: Default::default(),
                    owner_id: 2,
                }))
            });

        let mut request = Request::new(DeleteProjectRequest { id: 1 });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.delete_project(request).await.unwrap_err();

        assert_eq!(res.code(), Code::PermissionDenied);
    }

    #[tokio::test]
    async fn delete_invalid_project_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(2))
            .returning(move |_| Ok(None));

        let mut request = Request::new(DeleteProjectRequest { id: 2 });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.delete_project(request).await.unwrap_err();

        assert_eq!(res.code(), Code::NotFound);
    }

    #[tokio::test]
    async fn delete_project_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: 1,
                    name: Default::default(),
                    components_info: Default::default(),
                    owner_id: 1,
                }))
            });

        mock_contexts
            .project_context_mock
            .expect_delete()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(project::Model {
                    id: 1,
                    name: Default::default(),
                    components_info: Default::default(),
                    owner_id: 1,
                })
            });

        let mut request = Request::new(DeleteProjectRequest { id: 1 });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.delete_project(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn get_project_user_has_no_access_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        let project = project::Model {
            id: Default::default(),
            name: "project".to_string(),
            components_info: Default::default(),
            owner_id: 0,
        };

        let in_use = in_use::Model {
            project_id: Default::default(),
            session_id: 0,
            latest_activity: Default::default(),
        };

        let queries: Vec<query::Model> = vec![];

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(0), predicate::eq(0))
            .returning(move |_, _| Ok(None));

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(project.clone())));

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(in_use.clone())));

        mock_contexts
            .query_context_mock
            .expect_get_all_by_project_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(queries.clone()));

        let mut request = Request::new(GetProjectRequest { id: 0 });

        request.metadata_mut().insert("uid", "0".parse().unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.get_project(request).await.unwrap_err();

        assert!(res.code() == Code::PermissionDenied);
    }

    #[tokio::test]
    async fn get_project_is_in_use_is_true() {
        let mut mock_contexts = get_mock_contexts();

        let project = project::Model {
            id: Default::default(),
            name: "project".to_string(),
            components_info: Default::default(),
            owner_id: 0,
        };

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: 1,
            user_id: 1,
        };

        let in_use = in_use::Model {
            project_id: Default::default(),
            session_id: 0,
            latest_activity: Utc::now().naive_utc(),
        };

        let queries: Vec<query::Model> = vec![];

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(0), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(project.clone())));

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(in_use.clone())));

        mock_contexts
            .query_context_mock
            .expect_get_all_by_project_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(queries.clone()));

        let mut request = Request::new(GetProjectRequest { id: 0 });

        request.metadata_mut().insert("uid", "0".parse().unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.get_project(request).await;

        assert!(res.unwrap().get_ref().in_use);
    }

    #[tokio::test]
    async fn get_project_is_in_use_is_false() {
        let mut mock_contexts = get_mock_contexts();

        let project = project::Model {
            id: Default::default(),
            name: "project".to_string(),
            components_info: Default::default(),
            owner_id: 0,
        };

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: 1,
            user_id: 1,
        };

        let in_use = in_use::Model {
            project_id: 0,
            session_id: 0,
            latest_activity: Default::default(),
        };

        let updated_in_use = in_use::Model {
            project_id: 0,
            session_id: 1,
            latest_activity: Default::default(),
        };

        let session = session::Model {
            id: 0,
            refresh_token: "refresh_token".to_owned(),
            access_token: "access_token".to_owned(),
            updated_at: Default::default(),
            user_id: Default::default(),
        };

        let queries: Vec<query::Model> = vec![];

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(0), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(project.clone())));

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(in_use.clone())));

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .with(
                predicate::eq(TokenType::AccessToken),
                predicate::eq("access_token".to_owned()),
            )
            .returning(move |_, _| Ok(Some(session.clone())));

        mock_contexts
            .query_context_mock
            .expect_get_all_by_project_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(queries.clone()));

        mock_contexts
            .in_use_context_mock
            .expect_update()
            .returning(move |_| Ok(updated_in_use.clone()));

        let mut request = Request::new(GetProjectRequest { id: 0 });

        request
            .metadata_mut()
            .insert("authorization", "Bearer access_token".parse().unwrap());
        request.metadata_mut().insert("uid", "0".parse().unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.get_project(request).await;

        assert!(!res.unwrap().get_ref().in_use);
    }

    #[tokio::test]
    async fn get_project_project_has_no_queries_queries_are_empty() {
        let mut mock_contexts = get_mock_contexts();

        let project = project::Model {
            id: Default::default(),
            name: "project".to_string(),
            components_info: Default::default(),
            owner_id: 0,
        };

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: 1,
            user_id: 1,
        };

        let in_use = in_use::Model {
            project_id: Default::default(),
            session_id: 0,
            latest_activity: Utc::now().naive_utc(),
        };

        let queries: Vec<query::Model> = vec![];

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(0), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(project.clone())));

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(in_use.clone())));

        mock_contexts
            .query_context_mock
            .expect_get_all_by_project_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(queries.clone()));

        let mut request = Request::new(GetProjectRequest { id: 0 });

        request.metadata_mut().insert("uid", "0".parse().unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.get_project(request).await;

        assert!(res.unwrap().get_ref().queries.is_empty());
    }

    #[tokio::test]
    async fn get_project_query_has_no_result_query_is_empty() {
        let mut mock_contexts = get_mock_contexts();

        let project = project::Model {
            id: Default::default(),
            name: "project".to_string(),
            components_info: Default::default(),
            owner_id: 0,
        };

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: 1,
            user_id: 1,
        };

        let in_use = in_use::Model {
            project_id: Default::default(),
            session_id: 0,
            latest_activity: Utc::now().naive_utc(),
        };

        let query = query::Model {
            id: 0,
            project_id: 1,
            string: "query".to_owned(),
            result: None,
            outdated: false,
        };

        let queries: Vec<query::Model> = vec![query];

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(0), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(project.clone())));

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(in_use.clone())));

        mock_contexts
            .query_context_mock
            .expect_get_all_by_project_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(queries.clone()));

        let mut request = Request::new(GetProjectRequest { id: 0 });

        request.metadata_mut().insert("uid", "0".parse().unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.get_project(request).await;

        assert!(res.unwrap().get_ref().queries[0].result.is_empty());
    }

    #[tokio::test]
    async fn list_projects_info_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

        let project_info = ProjectInfo {
            project_id: 1,
            project_name: "project::Model name".to_owned(),
            project_owner_id: 1,
            user_role_on_project: "Editor".to_owned(),
        };

        mock_contexts
            .project_context_mock
            .expect_get_project_info_by_uid()
            .with(predicate::eq(1))
            .returning(move |_| Ok(vec![project_info.clone()]));

        let mut list_projects_info_request = Request::new(());

        list_projects_info_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic
            .list_projects_info(list_projects_info_request)
            .await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn list_projects_info_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .project_context_mock
            .expect_get_project_info_by_uid()
            .with(predicate::eq(1))
            .returning(move |_| Ok(vec![]));

        let mut list_projects_info_request = Request::new(());

        list_projects_info_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic
            .list_projects_info(list_projects_info_request)
            .await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn update_name_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

        let user_id = 1;
        let project_id = 1;
        let new_project_name = "new_name".to_string();

        let mut update_project_request = Request::new(UpdateProjectRequest {
            id: project_id,
            name: Some(new_project_name.clone()),
            components_info: None,
            owner_id: None,
        });

        update_project_request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
        );

        update_project_request.metadata_mut().insert(
            "uid",
            metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
        );

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(project_id))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: project_id,
                    name: "old_name".to_owned(),
                    components_info: Default::default(),
                    owner_id: user_id,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(project_id))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    user_id,
                    project_id,
                    role: "Editor".to_string(),
                }))
            });

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .with(
                predicate::eq(TokenType::AccessToken),
                predicate::eq("access_token".to_string()),
            )
            .returning(move |_, _| {
                Ok(Some(session::Model {
                    id: 1,
                    refresh_token: "refresh_token".to_string(),
                    access_token: "access_token".to_string(),
                    updated_at: Default::default(),
                    user_id,
                }))
            });

        mock_contexts
            .project_context_mock
            .expect_update()
            .returning(move |_| {
                Ok(project::Model {
                    id: project_id,
                    name: new_project_name.clone(),
                    components_info: Default::default(),
                    owner_id: user_id,
                })
            });

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .returning(move |_| {
                Ok(Some(in_use::Model {
                    project_id,
                    session_id: 1,
                    latest_activity: Utc::now().naive_utc(),
                }))
            });

        mock_contexts
            .in_use_context_mock
            .expect_update()
            .returning(move |_| {
                Ok(in_use::Model {
                    project_id: 1,
                    session_id: 1,
                    latest_activity: Utc::now().naive_utc(),
                })
            });

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.update_project(update_project_request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn update_components_info_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

        let user_id = 1;
        let project_id = 1;
        let components_info_non_json = ComponentsInfo {
            components: vec![Component {
                rep: Some(Rep::Json("a".to_owned())),
            }],
            components_hash: 1234456,
        };
        let components_info = serde_json::to_value(components_info_non_json.clone()).unwrap();

        let mut update_project_request = Request::new(UpdateProjectRequest {
            id: project_id,
            name: None,
            components_info: Some(components_info_non_json.clone()),
            owner_id: None,
        });

        update_project_request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
        );

        update_project_request.metadata_mut().insert(
            "uid",
            metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
        );

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(project_id))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: project_id,
                    name: Default::default(),
                    components_info: Default::default(),
                    owner_id: user_id,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(project_id))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    user_id,
                    project_id,
                    role: "Editor".to_string(),
                }))
            });

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .with(
                predicate::eq(TokenType::AccessToken),
                predicate::eq("access_token".to_string()),
            )
            .returning(move |_, _| {
                Ok(Some(session::Model {
                    id: 1,
                    refresh_token: "refresh_token".to_string(),
                    access_token: "access_token".to_string(),
                    updated_at: Default::default(),
                    user_id,
                }))
            });

        mock_contexts
            .project_context_mock
            .expect_update()
            .returning(move |_| {
                Ok(project::Model {
                    id: project_id,
                    name: Default::default(),
                    components_info: components_info.clone(),
                    owner_id: user_id,
                })
            });

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .returning(move |_| {
                Ok(Some(in_use::Model {
                    project_id,
                    session_id: 1,
                    latest_activity: Utc::now().naive_utc(),
                }))
            });

        mock_contexts
            .in_use_context_mock
            .expect_update()
            .returning(move |_| {
                Ok(in_use::Model {
                    project_id: 1,
                    session_id: 1,
                    latest_activity: Utc::now().naive_utc(),
                })
            });

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.update_project(update_project_request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn update_owner_id_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

        let user_id = 1;
        let project_id = 1;
        let new_owner_id = 2;

        let mut update_project_request = Request::new(UpdateProjectRequest {
            id: project_id,
            name: None,
            components_info: None,
            owner_id: Some(new_owner_id),
        });

        update_project_request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
        );

        update_project_request.metadata_mut().insert(
            "uid",
            metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
        );

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(project_id))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: project_id,
                    name: Default::default(),
                    components_info: Default::default(),
                    owner_id: user_id,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(project_id))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    user_id,
                    project_id,
                    role: "Editor".to_string(),
                }))
            });

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .with(
                predicate::eq(TokenType::AccessToken),
                predicate::eq("access_token".to_string()),
            )
            .returning(move |_, _| {
                Ok(Some(session::Model {
                    id: 1,
                    refresh_token: "refresh_token".to_string(),
                    access_token: "access_token".to_string(),
                    updated_at: Default::default(),
                    user_id,
                }))
            });

        mock_contexts
            .project_context_mock
            .expect_update()
            .returning(move |_| {
                Ok(project::Model {
                    id: project_id,
                    name: Default::default(),
                    components_info: Default::default(),
                    owner_id: new_owner_id,
                })
            });

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .returning(move |_| {
                Ok(Some(in_use::Model {
                    project_id,
                    session_id: 1,
                    latest_activity: Utc::now().naive_utc(),
                }))
            });

        mock_contexts
            .in_use_context_mock
            .expect_update()
            .returning(move |_| {
                Ok(in_use::Model {
                    project_id: 1,
                    session_id: 1,
                    latest_activity: Utc::now().naive_utc(),
                })
            });

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.update_project(update_project_request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn update_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

        let user_id = 1;
        let project_id = 1;
        let new_project_name = "new_name".to_string();
        let new_components_info_non_json = ComponentsInfo {
            components: vec![Component {
                rep: Some(Rep::Json("a".to_owned())),
            }],
            components_hash: 1234456,
        };
        let new_components_info =
            serde_json::to_value(new_components_info_non_json.clone()).unwrap();
        let new_owner_id = 2;

        let mut update_project_request = Request::new(UpdateProjectRequest {
            id: project_id,
            name: Some(new_project_name.clone()),
            components_info: Some(new_components_info_non_json.clone()),
            owner_id: Some(new_owner_id),
        });

        update_project_request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
        );

        update_project_request.metadata_mut().insert(
            "uid",
            metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
        );

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(project_id))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: project_id,
                    name: "old_name".to_owned(),
                    components_info: serde_json::to_value("{\"old_components\":1}").unwrap(),
                    owner_id: user_id,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(project_id))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    user_id,
                    project_id,
                    role: "Editor".to_string(),
                }))
            });

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .with(
                predicate::eq(TokenType::AccessToken),
                predicate::eq("access_token".to_string()),
            )
            .returning(move |_, _| {
                Ok(Some(session::Model {
                    id: 1,
                    refresh_token: "refresh_token".to_string(),
                    access_token: "access_token".to_string(),
                    updated_at: Default::default(),
                    user_id,
                }))
            });

        mock_contexts
            .project_context_mock
            .expect_update()
            .returning(move |_| {
                Ok(project::Model {
                    id: project_id,
                    name: new_project_name.clone(),
                    components_info: new_components_info.clone(),
                    owner_id: new_owner_id,
                })
            });

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .returning(move |_| {
                Ok(Some(in_use::Model {
                    project_id,
                    session_id: 1,
                    latest_activity: Utc::now().naive_utc(),
                }))
            });

        mock_contexts
            .in_use_context_mock
            .expect_update()
            .returning(move |_| {
                Ok(in_use::Model {
                    project_id: 1,
                    session_id: 1,
                    latest_activity: Utc::now().naive_utc(),
                })
            });

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.update_project(update_project_request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn update_owner_not_owner_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: 1,
                    name: Default::default(),
                    components_info: Default::default(),
                    owner_id: 2,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    user_id: 1,
                    project_id: 1,
                    role: "Editor".to_owned(),
                }))
            });

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .with(
                predicate::eq(TokenType::AccessToken),
                predicate::eq("access_token".to_string()),
            )
            .returning(move |_, _| {
                Ok(Some(session::Model {
                    id: 1,
                    refresh_token: "refresh_token".to_string(),
                    access_token: "access_token".to_string(),
                    updated_at: Default::default(),
                    user_id: 1,
                }))
            });

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(in_use::Model {
                    session_id: 1,
                    latest_activity: Default::default(),
                    project_id: 1,
                }))
            });

        mock_contexts
            .in_use_context_mock
            .expect_update()
            .returning(move |_| {
                Ok(in_use::Model {
                    session_id: 1,
                    latest_activity: Default::default(),
                    project_id: 1,
                })
            });

        let mut request = Request::new(UpdateProjectRequest {
            id: 1,
            name: None,
            components_info: None,
            owner_id: Some(1),
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str("access_token").unwrap(),
        );

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.update_project(request).await.unwrap_err();

        assert_eq!(res.code(), Code::PermissionDenied);
    }

    #[tokio::test]
    async fn update_no_in_use_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: 1,
                    name: Default::default(),
                    components_info: Default::default(),
                    owner_id: 1,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    user_id: 1,
                    project_id: 1,
                    role: "Editor".to_owned(),
                }))
            });

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .with(
                predicate::eq(TokenType::AccessToken),
                predicate::eq("access_token".to_string()),
            )
            .returning(move |_, _| {
                Ok(Some(session::Model {
                    id: 1,
                    refresh_token: "refresh_token".to_string(),
                    access_token: "access_token".to_string(),
                    updated_at: Default::default(),
                    user_id: 1,
                }))
            });

        mock_contexts
            .in_use_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(in_use::Model {
                    session_id: 2,
                    latest_activity: Utc::now().naive_utc(),
                    project_id: 1,
                }))
            });

        let mut request = Request::new(UpdateProjectRequest {
            id: 1,
            name: None,
            components_info: None,
            owner_id: None,
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str("access_token").unwrap(),
        );

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.update_project(request).await.unwrap_err();

        assert_eq!(res.code(), Code::FailedPrecondition);
    }

    #[tokio::test]
    async fn update_no_access_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: 1,
                    name: Default::default(),
                    components_info: Default::default(),
                    owner_id: 1,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| Ok(None));

        let mut request = Request::new(UpdateProjectRequest {
            id: 1,
            name: None,
            components_info: None,
            owner_id: None,
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.update_project(request).await.unwrap_err();

        assert_eq!(res.code(), Code::PermissionDenied);
    }

    #[tokio::test]
    async fn update_incorrect_role_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: 1,
                    name: Default::default(),
                    components_info: Default::default(),
                    owner_id: 1,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    user_id: 1,
                    project_id: 1,
                    role: "Viewer".to_owned(),
                }))
            });

        let mut request = Request::new(UpdateProjectRequest {
            id: 1,
            name: None,
            components_info: None,
            owner_id: None,
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.update_project(request).await.unwrap_err();

        assert_eq!(res.code(), Code::PermissionDenied);
    }

    #[tokio::test]
    async fn update_no_session_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: 1,
                    name: Default::default(),
                    components_info: Default::default(),
                    owner_id: 1,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    user_id: 1,
                    project_id: 1,
                    role: "Editor".to_owned(),
                }))
            });

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .with(
                predicate::eq(TokenType::AccessToken),
                predicate::eq("access_token".to_string()),
            )
            .returning(move |_, _| Ok(None));

        let mut request = Request::new(UpdateProjectRequest {
            id: 1,
            name: None,
            components_info: None,
            owner_id: None,
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str("access_token").unwrap(),
        );

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.update_project(request).await.unwrap_err();

        assert_eq!(res.code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn update_no_project_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(2))
            .returning(move |_| Ok(None));

        let mut request = Request::new(UpdateProjectRequest {
            id: 2,
            name: None,
            components_info: None,
            owner_id: None,
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let project_logic = ProjectController::new(contexts);

        let res = project_logic.update_project(request).await.unwrap_err();

        assert_eq!(res.code(), Code::NotFound);
    }
}
