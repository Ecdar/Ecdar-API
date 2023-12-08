use crate::api::auth::RequestExt;
use crate::api::server::protobuf::create_access_request::User;
use crate::api::server::protobuf::{
    CreateAccessRequest, DeleteAccessRequest, ListAccessInfoRequest, ListAccessInfoResponse,
    UpdateAccessRequest,
};
use crate::contexts::context_collection::ContextCollection;
use crate::contexts::context_traits::{AccessContextTrait, UserContextTrait};
use crate::controllers::controller_traits::AccessControllerTrait;
use crate::entities::{access, user};
use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Code, Request, Response, Status};

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
            .map_err(|_err| Status::internal("could not stringify user id in request metadata"))?
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

    /// Creates an access in the contexts.
    /// # Errors
    /// Returns an error if the contexts context fails to create the access
    async fn create_access(
        &self,
        request: Request<CreateAccessRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();

        let uid = request
            .uid()
            .map_err(|_err| Status::internal("could not stringify user id in request metadata"))?
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
    /// `project_id` and `user_id` is set to 'default' since they won't be updated in the contexts.
    async fn update_access(
        &self,
        request: Request<UpdateAccessRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();

        let uid = request
            .uid()
            .map_err(|_err| Status::internal("could not stringify user id in request metadata"))?
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

    /// Deletes the an Access from the contexts. This has no sideeffects.
    ///
    /// # Errors
    /// This function will return an error if the access does not exist in the contexts.
    async fn delete_access(
        &self,
        request: Request<DeleteAccessRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.get_ref().clone();

        let uid = request
            .uid()
            .map_err(|_err| Status::internal("could not stringify user id in request metadata"))?
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

#[cfg(test)]
#[path = "../../tests/controllers/access_controller.rs"]
mod access_controller_tests;
