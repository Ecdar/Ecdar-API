use crate::api::auth::RequestExt;
use crate::api::server::protobuf::{
    CreateQueryRequest, DeleteQueryRequest, QueryRequest, SendQueryRequest, SendQueryResponse,
    UpdateQueryRequest,
};
use crate::contexts::ContextCollection;
use crate::entities::query;
use crate::services::ServiceCollection;
use async_trait::async_trait;
use tonic::{Code, Request, Response, Status};

#[async_trait]
pub trait QueryControllerTrait: Send + Sync {
    /// Creates a query in the contexts
    /// # Errors
    /// Returns an error if the contexts context fails to create the query or
    async fn create_query(
        &self,
        request: Request<CreateQueryRequest>,
    ) -> Result<Response<()>, Status>;

    /// Endpoint for updating a query record.
    /// # Errors
    /// Errors on non existent entity, parsing error or invalid rights
    async fn update_query(
        &self,
        request: Request<UpdateQueryRequest>,
    ) -> Result<Response<()>, Status>;

    /// Deletes a query record in the contexts.
    /// # Errors
    /// Returns an error if the provided query_id is not found in the contexts.
    async fn delete_query(
        &self,
        request: Request<DeleteQueryRequest>,
    ) -> Result<Response<()>, Status>;

    /// Sends a query to be run on Reveaal.
    /// After query is run the result is stored in the contexts.
    ///
    /// Returns the response that is received from Reveaal.
    async fn send_query(
        &self,
        request: Request<SendQueryRequest>,
    ) -> Result<Response<SendQueryResponse>, Status>;
}

pub struct QueryController {
    contexts: ContextCollection,
    services: ServiceCollection,
}

impl QueryController {
    pub fn new(contexts: ContextCollection, services: ServiceCollection) -> Self {
        Self { contexts, services }
    }
}

#[async_trait]
impl QueryControllerTrait for QueryController {
    async fn create_query(
        &self,
        request: Request<CreateQueryRequest>,
    ) -> Result<Response<()>, Status> {
        let query_request = request.get_ref();

        let access = self
            .contexts
            .access_context
            .get_access_by_uid_and_project_id(
                request
                    .uid()
                    .map_err(|err| {
                        Status::invalid_argument(format!(
                            "could not stringify user id in request metadata, inner error {}",
                            err
                        ))
                    })?
                    .ok_or(Status::invalid_argument(
                        "failed to get user id from request metadata",
                    ))?,
                query_request.project_id,
            )
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
            .get_access_by_uid_and_project_id(
                request
                    .uid()
                    .map_err(|err| {
                        Status::internal(format!(
                            "could not stringify user id in request metadata, internal error {}",
                            err
                        ))
                    })?
                    .ok_or(Status::internal(
                        "failed to get user id from request metadata",
                    ))?,
                old_query.project_id,
            )
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
            .get_access_by_uid_and_project_id(
                request
                    .uid()
                    .map_err(|err| {
                        Status::internal(format!(
                            "could not stringify user id in request metadata, internal error {}",
                            err
                        ))
                    })?
                    .ok_or(Status::internal(
                        "failed to get user id from request metadata",
                    ))?,
                query.project_id,
            )
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

    async fn send_query(
        &self,
        request: Request<SendQueryRequest>,
    ) -> Result<Response<SendQueryResponse>, Status> {
        let message = request.get_ref();

        let uid = request
            .uid()
            .map_err(|err| {
                Status::internal(format!(
                    "could not stringify user id in request metadata, internal error {}",
                    err
                ))
            })?
            .ok_or(Status::internal(
                "failed to get user id from request metadata",
            ))?;

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

        // Get project from contexts
        let project = self
            .contexts
            .project_context
            .get_by_id(message.project_id)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::NotFound, "Model not found"))?;

        // Get query from contexts
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
            components_info: serde_json::from_value(project.components_info).map_err(|err| {
                Status::internal(format!(
                    "error parsing query result, internal error: {}",
                    err
                ))
            })?,
            settings: Default::default(), //TODO
        });

        // Run query on Reveaal
        let query_result = self
            .services
            .reveaal_service
            .send_query(query_request)
            .await?;

        // Update query result in contexts
        self.contexts
            .query_context
            .update(query::Model {
                id: query.id,
                string: query.string.clone(),
                result: Some(
                    serde_json::to_value(
                        query_result
                            .get_ref()
                            .result
                            .clone()
                            .ok_or(Status::internal("failed to get query result"))?, //TODO better error message ?
                    )
                    .map_err(|err| {
                        Status::internal(format!(
                            "error parsing query result, internal error: {}",
                            err
                        ))
                    })?,
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
}

#[cfg(test)]
mod tests {
    use super::super::helpers::{
        disguise_context_mocks, disguise_service_mocks, get_mock_contexts, get_mock_services,
    };
    use crate::api::server::protobuf::query_response::{self, Result};
    use crate::api::server::protobuf::{
        CreateQueryRequest, DeleteQueryRequest, QueryResponse, SendQueryRequest, UpdateQueryRequest,
    };
    use crate::controllers::QueryController;
    use crate::controllers::QueryControllerTrait;
    use crate::entities::{access, project, query};
    use mockall::predicate;
    use sea_orm::DbErr;
    use std::str::FromStr;
    use tonic::{metadata, Code, Request, Response};

    #[tokio::test]
    async fn create_invalid_query_returns_err() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let query = query::Model {
            id: Default::default(),
            string: "".to_string(),
            result: Default::default(),
            project_id: 1,
            outdated: Default::default(),
        };

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: 1,
            user_id: 1,
        };

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .query_context_mock
            .expect_create()
            .with(predicate::eq(query.clone()))
            .returning(move |_| Err(DbErr::RecordNotInserted));

        let mut request = Request::new(CreateQueryRequest {
            string: "".to_string(),
            project_id: 1,
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let query_logic = QueryController::new(contexts, services);

        let res = query_logic.create_query(request).await.unwrap_err();

        assert_eq!(res.code(), Code::Internal);
    }

    #[tokio::test]
    async fn create_query_returns_ok() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let query = query::Model {
            id: Default::default(),
            string: "".to_string(),
            result: Default::default(),
            project_id: 1,
            outdated: Default::default(),
        };

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: 1,
            user_id: 1,
        };

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .query_context_mock
            .expect_create()
            .with(predicate::eq(query.clone()))
            .returning(move |_| Ok(query.clone()));

        let mut request = Request::new(CreateQueryRequest {
            string: "".to_string(),
            project_id: 1,
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let query_logic = QueryController::new(contexts, services);

        let res = query_logic.create_query(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn update_invalid_query_returns_err() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let old_query = query::Model {
            id: 1,
            string: "".to_string(),
            result: None,
            project_id: Default::default(),
            outdated: true,
        };

        let query = query::Model {
            string: "updated".to_string(),
            ..old_query.clone()
        };

        let access = access::Model {
            id: 1,
            role: "Editor".to_string(),
            project_id: Default::default(),
            user_id: 1,
        };

        mock_contexts
            .query_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| Ok(Some(old_query.clone())));

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .query_context_mock
            .expect_update()
            .with(predicate::eq(query.clone()))
            .returning(move |_| Err(DbErr::RecordNotUpdated));

        let mut request = Request::new(UpdateQueryRequest {
            id: 1,
            string: "updated".to_string(),
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let query_logic = QueryController::new(contexts, services);

        let res = query_logic.update_query(request).await.unwrap_err();

        assert_eq!(res.code(), Code::Internal);
    }

    #[tokio::test]
    async fn update_query_returns_ok() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let old_query = query::Model {
            id: 1,
            string: "".to_string(),
            result: None,
            project_id: Default::default(),
            outdated: true,
        };

        let query = query::Model {
            string: "updated".to_string(),
            ..old_query.clone()
        };

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: Default::default(),
            user_id: 1,
        };

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .query_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| Ok(Some(old_query.clone())));

        mock_contexts
            .query_context_mock
            .expect_update()
            .with(predicate::eq(query.clone()))
            .returning(move |_| Ok(query.clone()));

        let mut request = Request::new(UpdateQueryRequest {
            id: 1,
            string: "updated".to_string(),
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let query_logic = QueryController::new(contexts, services);

        let res = query_logic.update_query(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn delete_invalid_query_returns_err() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: Default::default(),
            user_id: 1,
        };

        let query = query::Model {
            id: 1,
            string: "".to_string(),
            result: Default::default(),
            project_id: Default::default(),
            outdated: Default::default(),
        };

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .query_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| Ok(Some(query.clone())));

        mock_contexts
            .query_context_mock
            .expect_delete()
            .with(predicate::eq(1))
            .returning(move |_| Err(DbErr::RecordNotFound("".to_string())));

        let mut request = Request::new(DeleteQueryRequest { id: 1 });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let query_logic = QueryController::new(contexts, services);

        let res = query_logic.delete_query(request).await.unwrap_err();

        assert_eq!(res.code(), Code::NotFound);
    }

    #[tokio::test]
    async fn delete_query_returns_ok() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let query = query::Model {
            id: 1,
            string: "".to_string(),
            result: Default::default(),
            project_id: Default::default(),
            outdated: Default::default(),
        };

        let query_clone = query.clone();

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: Default::default(),
            user_id: 1,
        };

        mock_contexts
            .query_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| Ok(Some(query.clone())));

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .query_context_mock
            .expect_delete()
            .with(predicate::eq(1))
            .returning(move |_| Ok(query_clone.clone()));

        let mut request = Request::new(DeleteQueryRequest { id: 1 });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let query_logic = QueryController::new(contexts, services);

        let res = query_logic.delete_query(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn create_query_invalid_role_returns_err() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let query = query::Model {
            id: 1,
            string: "".to_string(),
            result: Default::default(),
            project_id: Default::default(),
            outdated: Default::default(),
        };

        let access = access::Model {
            id: Default::default(),
            role: "Viewer".to_string(),
            project_id: Default::default(),
            user_id: 1,
        };

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .query_context_mock
            .expect_create()
            .with(predicate::eq(query.clone()))
            .returning(move |_| Ok(query.clone()));

        let mut request = Request::new(CreateQueryRequest {
            string: "".to_string(),
            project_id: 1,
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let query_logic = QueryController::new(contexts, services);

        let res = query_logic.create_query(request).await.unwrap_err();

        assert_eq!(res.code(), Code::PermissionDenied);
    }

    #[tokio::test]
    async fn delete_query_invalid_role_returns_err() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let query = query::Model {
            id: 1,
            string: "".to_string(),
            result: Default::default(),
            project_id: Default::default(),
            outdated: Default::default(),
        };

        let query_clone = query.clone();

        let access = access::Model {
            id: Default::default(),
            role: "Viewer".to_string(),
            project_id: Default::default(),
            user_id: 1,
        };

        mock_contexts
            .query_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| Ok(Some(query.clone())));

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .query_context_mock
            .expect_delete()
            .with(predicate::eq(1))
            .returning(move |_| Ok(query_clone.clone()));

        let mut request = Request::new(DeleteQueryRequest { id: 1 });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let query_logic = QueryController::new(contexts, services);

        let res = query_logic.delete_query(request).await.unwrap_err();

        assert_eq!(res.code(), Code::PermissionDenied);
    }

    #[tokio::test]
    async fn update_query_invalid_role_returns_err() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let old_query = query::Model {
            id: 1,
            string: "".to_string(),
            result: None,
            project_id: Default::default(),
            outdated: true,
        };

        let query = query::Model {
            string: "updated".to_string(),
            ..old_query.clone()
        };

        let access = access::Model {
            id: Default::default(),
            role: "Viewer".to_string(),
            project_id: Default::default(),
            user_id: 1,
        };

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .query_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| Ok(Some(old_query.clone())));

        mock_contexts
            .query_context_mock
            .expect_update()
            .with(predicate::eq(query.clone()))
            .returning(move |_| Ok(query.clone()));

        let mut request = Request::new(UpdateQueryRequest {
            id: 1,
            string: "updated".to_string(),
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let query_logic = QueryController::new(contexts, services);

        let res = query_logic.update_query(request).await.unwrap_err();

        assert_eq!(res.code(), Code::PermissionDenied);
    }

    #[tokio::test]
    async fn send_query_returns_ok() {
        let mut mock_contexts = get_mock_contexts();
        let mut mock_services = get_mock_services();

        let query = query::Model {
            id: Default::default(),
            string: "".to_string(),
            result: Default::default(),
            project_id: Default::default(),
            outdated: Default::default(),
        };

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: Default::default(),
            user_id: 1,
        };

        let project = project::Model {
            id: Default::default(),
            name: "project".to_string(),
            components_info: Default::default(),
            owner_id: 0,
        };

        let query_response = QueryResponse {
            query_id: Default::default(),
            info: Default::default(),
            result: Some(Result::Success(query_response::Success {})),
        };

        let updated_query = query::Model {
            result: Some(serde_json::to_value(query_response.clone().result).unwrap()),
            ..query.clone()
        };

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(project.clone())));

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(0))
            .returning(move |_, _| Ok(Some(access.clone())));

        mock_contexts
            .query_context_mock
            .expect_get_by_id()
            .with(predicate::eq(0))
            .returning(move |_| Ok(Some(query.clone())));

        mock_services
            .reveaal_service_mock
            .expect_send_query()
            .returning(move |_| Ok(Response::new(query_response.clone())));

        mock_contexts
            .query_context_mock
            .expect_update()
            .with(predicate::eq(updated_query.clone()))
            .returning(move |_| Ok(updated_query.clone()));

        let mut request = Request::new(SendQueryRequest {
            id: Default::default(),
            project_id: Default::default(),
        });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let query_logic = QueryController::new(contexts, services);

        let res = query_logic.send_query(request).await;

        assert!(res.is_ok());
    }
}
