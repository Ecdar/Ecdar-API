use crate::api::auth::RequestExt;
use crate::api::server::protobuf::create_access_request::User;
use crate::api::server::protobuf::{
    CreateAccessRequest, DeleteAccessRequest, ListAccessInfoRequest, ListAccessInfoResponse,
    UpdateAccessRequest,
};
use crate::contexts::{AccessContextTrait, ContextCollection, UserContextTrait};
use crate::entities::{access, user};
use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[async_trait]
pub trait AccessControllerTrait: Send + Sync {
    /// handles the list_access_info endpoint
    /// # Errors
    /// If an invalid or non-existent [`ListAccessInfoRequest::project_id`] is provided
    async fn list_access_info(
        &self,
        request: Request<ListAccessInfoRequest>,
    ) -> Result<Response<ListAccessInfoResponse>, Status>;
    /// Creates access in the contexts.
    /// # Errors
    /// Returns an error if the contexts context fails to create the access
    async fn create_access(
        &self,
        request: Request<CreateAccessRequest>,
    ) -> Result<Response<()>, Status>;

    /// Endpoint for updating an access record.
    ///
    /// Takes [`UpdateAccessRequest`] as input
    ///
    /// Returns a [`Status`] as response
    ///
    /// `project_id` and `user_id` is set to 'default' since they won't be updated in the contexts.
    async fn update_access(
        &self,
        request: Request<UpdateAccessRequest>,
    ) -> Result<Response<()>, Status>;

    /// Deletes the an Access from the contexts. This has no sideeffects.
    ///
    /// # Errors
    /// This function will return an error if the access does not exist in the contexts.
    async fn delete_access(
        &self,
        request: Request<DeleteAccessRequest>,
    ) -> Result<Response<()>, Status>;
}

pub struct AccessController {
    contexts: ContextCollection,
}

impl AccessController {
    pub fn new(contexts: ContextCollection) -> Self {
        AccessController { contexts }
    }
}
#[async_trait]
impl AccessControllerTrait for AccessController {
    async fn list_access_info(
        &self,
        request: Request<ListAccessInfoRequest>,
    ) -> Result<Response<ListAccessInfoResponse>, Status> {
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

        self.contexts
            .access_context
            .get_access_by_uid_and_project_id(uid, message.project_id)
            .await
            .map_err(|error| Status::internal(error.to_string()))?
            .ok_or(Status::permission_denied(
                "User does not have access to model",
            ))?;
        self.contexts
            .access_context
            .get_access_by_project_id(message.project_id)
            .await
            .map_err(|error| Status::internal(error.to_string()))
            .and_then(|access_info_list| {
                if access_info_list.is_empty() {
                    Err(Status::not_found("No access found for given user"))
                } else {
                    Ok(Response::new(ListAccessInfoResponse { access_info_list }))
                }
            })
    }

    async fn create_access(
        &self,
        request: Request<CreateAccessRequest>,
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
                Err(error) => Err(Status::internal(error.to_string())),
            }
        } else {
            Err(Status::invalid_argument("No user identification provided"))
        }
    }

    async fn update_access(
        &self,
        request: Request<UpdateAccessRequest>,
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

        let user_access = self
            .contexts
            .access_context
            .get_by_id(message.id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or_else(|| Status::not_found("No access entity found for user".to_string()))?;

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
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or_else(|| Status::not_found("No model found for access".to_string()))?;

        // Check that the requester is not trying to update the owner's access
        if model.owner_id == message.id {
            return Err(Status::permission_denied(
                "Requester does not have permission to update access for this user",
            ));
        }

        let access = access::Model {
            id: message.id,
            role: message.role,
            project_id: Default::default(),
            user_id: Default::default(),
        };

        self.contexts
            .access_context
            .update(access)
            .await
            .map(|_| Response::new(()))
            .map_err(|error| Status::internal(error.to_string()))
    }

    async fn delete_access(
        &self,
        request: Request<DeleteAccessRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();

        let uid = request
            .uid()
            .map_err(|err| {
                Status::internal(format!(
                    "could not stringify user id in request metadata, inner error: {}",
                    err
                ))
            })?
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        let user_access = self
            .contexts
            .access_context
            .get_by_id(message.id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or_else(|| Status::not_found("No access entity found for user".to_string()))?;

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
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or_else(|| Status::not_found("No model found for access".to_string()))?;

        // Check that the requester is not trying to delete the owner's access
        if model.owner_id == message.id {
            return Err(Status::permission_denied(
                "You cannot delete the access entity for this user",
            ));
        }

        match self.contexts.access_context.delete(message.id).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => match error {
                sea_orm::DbErr::RecordNotFound(message) => Err(Status::not_found(message)),
                _ => Err(Status::internal(error.to_string())),
            },
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
        .map_err(|err| Status::internal(err.to_string()))?
        .ok_or_else(|| {
            Status::permission_denied("User does not have access to model".to_string())
        })?;

    // Check if the requester has role 'Editor'
    if access.role != "Editor" {
        Err(Status::permission_denied(
            "User does not have 'Editor' role for this model",
        ))
    } else {
        Ok(())
    }
}

async fn create_access_find_user_helper(
    user_context: Arc<dyn UserContextTrait>,
    user: User,
) -> Result<user::Model, Status> {
    match user {
        User::UserId(user_id) => Ok(user_context
            .get_by_id(user_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or_else(|| Status::not_found("No user found with given id"))?),

        User::Username(username) => Ok(user_context
            .get_by_username(username)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or_else(|| Status::not_found("No user found with given username"))?),

        User::Email(email) => Ok(user_context
            .get_by_email(email)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or_else(|| Status::not_found("No user found with given email"))?),
    }
}

#[cfg(test)]
mod tests {
    use super::super::helpers::{disguise_context_mocks, get_mock_contexts};
    use crate::api::server::protobuf::create_access_request::User;
    use crate::api::server::protobuf::{
        AccessInfo, CreateAccessRequest, DeleteAccessRequest, ListAccessInfoRequest,
        UpdateAccessRequest,
    };
    use crate::controllers::AccessController;
    use crate::controllers::AccessControllerTrait;
    use crate::entities::{access, project, user};
    use mockall::predicate;
    use sea_orm::DbErr;
    use std::str::FromStr;
    use tonic::{metadata, Code, Request};

    #[tokio::test]
    async fn create_invalid_access_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            project_id: 1,
            user_id: 1,
        };

        mock_contexts
            .access_context_mock
            .expect_create()
            .with(predicate::eq(access.clone()))
            .returning(move |_| Err(DbErr::RecordNotInserted));

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: Default::default(),
                    role: "Editor".to_owned(),
                    user_id: 1,
                    project_id: 1,
                }))
            });

        mock_contexts
            .user_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(user::Model {
                    id: 1,
                    email: Default::default(),
                    username: "test".to_string(),
                    password: "test".to_string(),
                }))
            });

        let mut request = Request::new(CreateAccessRequest {
            role: "Editor".to_string(),
            project_id: 1,
            user: Some(User::UserId(1)),
        });

        request.metadata_mut().insert(
            "uid",
            tonic::metadata::MetadataValue::from_str("1").unwrap(),
        );

        let contexts = disguise_context_mocks(mock_contexts);
        let access_logic = AccessController::new(contexts);

        let res = access_logic.create_access(request).await.unwrap_err();

        assert_eq!(res.code(), Code::Internal);
    }

    #[tokio::test]
    async fn create_access_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

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
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: Default::default(),
                    role: "Editor".to_string(),
                    user_id: 1,
                    project_id: 1,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_create()
            .with(predicate::eq(access.clone()))
            .returning(move |_| Ok(access.clone()));

        mock_contexts
            .user_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(user::Model {
                    id: 1,
                    email: Default::default(),
                    username: "test".to_string(),
                    password: "test".to_string(),
                }))
            });

        let mut request = Request::new(CreateAccessRequest {
            role: "Editor".to_string(),
            project_id: 1,
            user: Some(User::UserId(1)),
        });

        request.metadata_mut().insert(
            "uid",
            tonic::metadata::MetadataValue::from_str("1").unwrap(),
        );

        let contexts = disguise_context_mocks(mock_contexts);
        let access_logic = AccessController::new(contexts);

        let res = access_logic.create_access(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn update_invalid_access_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        let access = access::Model {
            id: 2,
            role: "Editor".to_string(),
            project_id: Default::default(),
            user_id: Default::default(),
        };

        mock_contexts
            .access_context_mock
            .expect_update()
            .with(predicate::eq(access.clone()))
            .returning(move |_| Err(DbErr::RecordNotUpdated));

        mock_contexts
            .access_context_mock
            .expect_get_by_id()
            .with(predicate::eq(2))
            .returning(move |_| {
                Ok(Some(access::Model {
                    id: 1,
                    role: "Editor".to_string(),
                    project_id: 1,
                    user_id: 2,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    role: "Editor".to_string(),
                    project_id: 1,
                    user_id: 1,
                }))
            });

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: 1,
                    name: "test".to_string(),
                    owner_id: 1,
                    components_info: Default::default(),
                }))
            });

        let mut request = Request::new(UpdateAccessRequest {
            id: 2,
            role: "Editor".to_string(),
        });

        request.metadata_mut().insert(
            "uid",
            tonic::metadata::MetadataValue::from_str("1").unwrap(),
        );

        let contexts = disguise_context_mocks(mock_contexts);
        let access_logic = AccessController::new(contexts);

        let res = access_logic.update_access(request).await.unwrap_err();

        assert_eq!(res.code(), Code::Internal);
    }

    #[tokio::test]
    async fn update_access_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

        let access = access::Model {
            id: 2,
            role: "Editor".to_string(),
            project_id: Default::default(),
            user_id: Default::default(),
        };

        mock_contexts
            .access_context_mock
            .expect_update()
            .with(predicate::eq(access.clone()))
            .returning(move |_| Ok(access.clone()));

        mock_contexts
            .access_context_mock
            .expect_get_by_id()
            .with(predicate::eq(2))
            .returning(move |_| {
                Ok(Some(access::Model {
                    id: 1,
                    role: "Editor".to_string(),
                    project_id: 1,
                    user_id: 2,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    role: "Editor".to_string(),
                    project_id: 1,
                    user_id: 1,
                }))
            });

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: 1,
                    name: "test".to_string(),
                    owner_id: 1,
                    components_info: Default::default(),
                }))
            });

        let mut request = Request::new(UpdateAccessRequest {
            id: 2,
            role: "Editor".to_string(),
        });

        request.metadata_mut().insert(
            "uid",
            tonic::metadata::MetadataValue::from_str("1").unwrap(),
        );

        let contexts = disguise_context_mocks(mock_contexts);
        let access_logic = AccessController::new(contexts);

        let res = access_logic.update_access(request).await;

        print!("{:?}", res);

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn delete_invalid_access_returns_err() {
        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .access_context_mock
            .expect_delete()
            .with(predicate::eq(2))
            .returning(move |_| Err(DbErr::RecordNotFound("".to_string())));

        mock_contexts
            .access_context_mock
            .expect_get_by_id()
            .with(predicate::eq(2))
            .returning(move |_| {
                Ok(Some(access::Model {
                    id: 1,
                    role: "Editor".to_string(),
                    project_id: 1,
                    user_id: 2,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    role: "Editor".to_string(),
                    project_id: 1,
                    user_id: 1,
                }))
            });

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: 1,
                    name: "test".to_string(),
                    owner_id: 1,
                    components_info: Default::default(),
                }))
            });

        let mut request = Request::new(DeleteAccessRequest { id: 2 });

        request.metadata_mut().insert(
            "uid",
            tonic::metadata::MetadataValue::from_str("1").unwrap(),
        );

        let contexts = disguise_context_mocks(mock_contexts);
        let access_logic = AccessController::new(contexts);

        let res = access_logic.delete_access(request).await.unwrap_err();

        assert_eq!(res.code(), Code::NotFound);
    }

    #[tokio::test]
    async fn delete_access_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

        let access = access::Model {
            id: 2,
            role: "Editor".to_string(),
            project_id: Default::default(),
            user_id: Default::default(),
        };

        mock_contexts
            .access_context_mock
            .expect_delete()
            .with(predicate::eq(2))
            .returning(move |_| Ok(access.clone()));

        mock_contexts
            .access_context_mock
            .expect_get_by_id()
            .with(predicate::eq(2))
            .returning(move |_| {
                Ok(Some(access::Model {
                    id: 1,
                    role: "Editor".to_string(),
                    project_id: 1,
                    user_id: 2,
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .with(predicate::eq(1), predicate::eq(1))
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    role: "Editor".to_string(),
                    project_id: 1,
                    user_id: 1,
                }))
            });

        mock_contexts
            .project_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| {
                Ok(Some(project::Model {
                    id: 1,
                    name: "test".to_string(),
                    owner_id: 1,
                    components_info: Default::default(),
                }))
            });

        let mut request = Request::new(DeleteAccessRequest { id: 2 });

        request.metadata_mut().insert(
            "uid",
            tonic::metadata::MetadataValue::from_str("1").unwrap(),
        );

        let contexts = disguise_context_mocks(mock_contexts);
        let access_logic = AccessController::new(contexts);

        let res = access_logic.delete_access(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn list_access_info_returns_ok() {
        let mut mock_contexts = get_mock_contexts();

        let mut request: Request<ListAccessInfoRequest> =
            Request::new(ListAccessInfoRequest { project_id: 1 });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let access = AccessInfo {
            id: 1,
            role: "Editor".to_string(),
            project_id: 1,
            user_id: 1,
        };

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .returning(move |_, _| {
                Ok(Some(access::Model {
                    id: 1,
                    role: "Editor".to_string(),
                    project_id: Default::default(),
                    user_id: Default::default(),
                }))
            });

        mock_contexts
            .access_context_mock
            .expect_get_access_by_project_id()
            .returning(move |_| Ok(vec![access.clone()]));

        let contexts = disguise_context_mocks(mock_contexts);
        let access_logic = AccessController::new(contexts);

        let res = access_logic.list_access_info(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn list_access_info_returns_not_found() {
        let mut mock_contexts = get_mock_contexts();

        let mut request = Request::new(ListAccessInfoRequest { project_id: 1 });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let access = access::Model {
            id: 1,
            role: "Editor".to_string(),
            project_id: 1,
            user_id: 1,
        };

        mock_contexts
            .access_context_mock
            .expect_get_access_by_project_id()
            .returning(move |_| Ok(vec![]));

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .returning(move |_, _| Ok(Some(access.clone())));

        let contexts = disguise_context_mocks(mock_contexts);
        let access_logic = AccessController::new(contexts);

        let res = access_logic.list_access_info(request).await.unwrap_err();

        assert_eq!(res.code(), Code::NotFound);
    }

    #[tokio::test]
    async fn list_access_info_returns_no_permission() {
        let mut request = Request::new(ListAccessInfoRequest { project_id: 1 });

        request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let mut mock_contexts = get_mock_contexts();

        mock_contexts
            .access_context_mock
            .expect_get_access_by_uid_and_project_id()
            .returning(move |_, _| Ok(None));

        let contexts = disguise_context_mocks(mock_contexts);
        let access_logic = AccessController::new(contexts);

        let res = access_logic.list_access_info(request).await.unwrap_err();

        assert_eq!(res.code(), Code::PermissionDenied);
    }
}
