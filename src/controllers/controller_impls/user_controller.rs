use crate::api::auth::RequestExt;
use crate::api::collections::{ContextCollection, ServiceCollection};
use crate::api::server::server::get_users_response::UserInfo;
use crate::api::server::server::{
    CreateUserRequest, GetUsersRequest, GetUsersResponse, UpdateUserRequest,
};
use crate::controllers::controller_traits::UserControllerTrait;
use crate::entities::user;
use regex::Regex;
use sea_orm::SqlErr;
use tonic::{Code, Request, Response, Status};

pub struct UserController {
    contexts: ContextCollection,
    services: ServiceCollection,
}

impl UserController {
    pub fn new(contexts: ContextCollection, services: ServiceCollection) -> Self {
        UserController { contexts, services }
    }
}

impl UserControllerTrait for UserController {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.into_inner().clone();

        if is_valid_username(message.clone().username.as_str()) {
            return Err(Status::new(Code::InvalidArgument, "Invalid username"));
        }

        if is_valid_email(message.clone().email.as_str()) {
            return Err(Status::new(Code::InvalidArgument, "Invalid email"));
        }

        let hashed_password = self
            .services
            .hashing_service
            .hash_password(message.clone().password);

        let user = user::Model {
            id: Default::default(),
            username: message.clone().username,
            password: hashed_password,
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
                Some(password) => self.services.hashing_service.hash_password(password),
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

    /// Gets users from the database.
    /// If no users exits with the given ids, an empty list is returned.
    async fn get_users(
        &self,
        request: Request<GetUsersRequest>,
    ) -> Result<Response<GetUsersResponse>, Status> {
        let ids = request.get_ref().ids.clone();

        let users = self
            .contexts
            .user_context
            .get_by_ids(ids)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let users_info = users
            .into_iter()
            .map(|user| UserInfo {
                id: user.id,
                username: user.username,
            })
            .collect::<Vec<UserInfo>>();

        Ok(Response::new(GetUsersResponse { users: users_info }))
    }
}

/// Returns true if the given email is a valid format.
fn is_valid_email(email: &str) -> bool {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .unwrap()
        .is_match(email)
}

/// Returns true if the given username is a valid format, i.e. only contains letters and numbers and a length from 3 to 32.
fn is_valid_username(username: &str) -> bool {
    Regex::new(r"^[a-zA-Z0-9_]{3,32}$")
        .unwrap()
        .is_match(username)
}

#[cfg(test)]
#[path = "../../tests/controllers/user_controller.rs"]
mod user_controller_tests;
