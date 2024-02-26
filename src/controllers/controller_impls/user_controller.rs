use crate::api::auth::RequestExt;
use crate::api::server::protobuf::get_users_response::UserInfo;
use crate::api::server::protobuf::{
    CreateUserRequest, GetUsersRequest, GetUsersResponse, UpdateUserRequest,
};
use crate::contexts::context_collection::ContextCollection;
use crate::controllers::controller_traits::UserControllerTrait;
use crate::entities::user;
use crate::services::service_collection::ServiceCollection;
use async_trait::async_trait;
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

    /// Returns true if the given email is a valid format.
    #[allow(clippy::expect_used)]
    fn is_valid_email(&self, email: &str) -> bool {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .expect("failed to compile regex")
            .is_match(email)
    }

    /// Returns true if the given username is a valid format, i.e. only contains letters and numbers and a length from 3 to 32.
    #[allow(clippy::expect_used)]
    fn is_valid_username(&self, username: &str) -> bool {
        Regex::new(r"^[a-zA-Z0-9_]{3,32}$")
            .expect("failed to compile regex")
            .is_match(username)
    }
}

#[async_trait]
impl UserControllerTrait for UserController {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.into_inner().clone();

        if !self.is_valid_username(message.clone().username.as_str()) {
            return Err(Status::new(Code::InvalidArgument, "Invalid username"));
        }

        if !self.is_valid_email(message.clone().email.as_str()) {
            return Err(Status::new(Code::InvalidArgument, "Invalid email"));
        }

        let hashed_password = self
            .services
            .hashing_service
            .hash_password(message.clone().password)
            .map_err(|_err| Status::internal("failed to hash password"))?;

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

    /// Updates a user record in the contexts.
    /// # Errors
    /// Returns an error if the contexts context fails to update the user or
    /// if the uid could not be parsed from the request metadata.
    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
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

        // Get user from contexts
        let user = self
            .contexts
            .user_context
            .get_by_id(uid)
            .await
            .map_err(|err| Status::new(Code::Internal, err.to_string()))?
            .ok_or_else(|| Status::new(Code::Internal, "No user found with given uid"))?;

        // Record to be inserted in contexts
        let new_user = user::Model {
            id: uid,
            username: match message.clone().username {
                Some(username) => {
                    if self.is_valid_username(username.as_str()) {
                        username
                    } else {
                        return Err(Status::new(Code::InvalidArgument, "Invalid username"));
                    }
                }
                None => user.username,
            },
            email: match message.clone().email {
                Some(email) => {
                    if self.is_valid_email(email.as_str()) {
                        email
                    } else {
                        return Err(Status::new(Code::InvalidArgument, "Invalid email"));
                    }
                }
                None => user.email,
            },
            password: match message.clone().password {
                Some(password) => self
                    .services
                    .hashing_service
                    .hash_password(password)
                    .map_err(|_err| Status::internal("failed to hash password"))?,
                None => user.password,
            },
        };

        // Update user in contexts
        match self.contexts.user_context.update(new_user).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    /// Deletes a user from the contexts.
    /// # Errors
    /// Returns an error if the contexts context fails to delete the user or
    /// if the uid could not be parsed from the request metadata.
    async fn delete_user(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let uid = request
            .uid()
            .map_err(|err| {
                Status::internal(format!(
                    "could not stringify user id in request metadata, internal error {}",
                    err
                ))
            })?
            .ok_or(Status::internal("Could not get uid from request metadata"))?;

        // Delete user from contexts
        match self.contexts.user_context.delete(uid).await {
            Ok(_) => Ok(Response::new(())),
            Err(error) => Err(Status::new(Code::Internal, error.to_string())),
        }
    }

    /// Gets users from the contexts.
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

#[cfg(test)]
mod tests {
    use super::super::helpers::{
        disguise_context_mocks, disguise_service_mocks, get_mock_contexts, get_mock_services,
    };
    use crate::api::server::protobuf::{CreateUserRequest, GetUsersRequest, UpdateUserRequest};
    use crate::controllers::controller_impls::UserController;
    use crate::controllers::controller_traits::UserControllerTrait;
    use crate::entities::user;
    use mockall::predicate;
    use sea_orm::DbErr;
    use std::str::FromStr;
    use tonic::{metadata, Code, Request};

    #[tokio::test]
    async fn delete_user_nonexistent_user_returns_err() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        mock_contexts
            .user_context_mock
            .expect_delete()
            .with(predicate::eq(1))
            .returning(|_| Err(DbErr::RecordNotFound("".into())));

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let mut delete_request = Request::new(());

        // Insert uid into request metadata
        delete_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let delete_response = user_logic.delete_user(delete_request).await.unwrap_err();
        let expected_response_code = Code::Internal;

        assert_eq!(delete_response.code(), expected_response_code);
    }

    #[tokio::test]
    async fn delete_user_existing_user_returns_ok() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let user = user::Model {
            id: 1,
            email: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
        };

        mock_contexts
            .user_context_mock
            .expect_delete()
            .with(predicate::eq(1))
            .returning(move |_| Ok(user.clone()));

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let mut delete_request = Request::new(());

        // Insert uid into request metadata
        delete_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let delete_response = user_logic.delete_user(delete_request).await;

        assert!(delete_response.is_ok());
    }

    #[tokio::test]
    async fn create_user_nonexistent_user_returns_ok() {
        let mut mock_contexts = get_mock_contexts();
        let mut mock_services = get_mock_services();

        let password = "Password123".to_string();

        let user = user::Model {
            id: Default::default(),
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: password.clone(),
        };

        let create_user_request = Request::new(CreateUserRequest {
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: password.clone(),
        });

        mock_services
            .hashing_service_mock
            .expect_hash_password()
            .returning(move |_| Ok(password.clone()));

        mock_contexts
            .user_context_mock
            .expect_create()
            .with(predicate::eq(user.clone()))
            .returning(move |_| Ok(user.clone()));

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let create_user_response = user_logic.create_user(create_user_request).await;
        assert!(create_user_response.is_ok());
    }

    #[tokio::test]
    async fn create_user_duplicate_email_returns_error() {
        let mut mock_contexts = get_mock_contexts();
        let mut mock_services = get_mock_services();

        let password = "Password123".to_string();

        let user = user::Model {
            id: Default::default(),
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: password.clone(),
        };

        let create_user_request = Request::new(CreateUserRequest {
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: password.clone(),
        });

        mock_services
            .hashing_service_mock
            .expect_hash_password()
            .returning(move |_| Ok(password.clone()));

        mock_contexts
            .user_context_mock
            .expect_create()
            .with(predicate::eq(user.clone()))
            .returning(move |_| Err(DbErr::RecordNotInserted)); //todo!("Needs to be a SqlError with UniqueConstraintViolation with 'email' in message)

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let res = user_logic.create_user(create_user_request).await;
        assert_eq!(res.unwrap_err().code(), Code::Internal); //todo!("Needs to be code AlreadyExists when mocked Error is corrected)
    }

    #[tokio::test]
    async fn create_user_invalid_email_returns_error() {
        let mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let create_user_request = Request::new(CreateUserRequest {
            email: "invalid-email".to_string(),
            username: "newuser".to_string(),
            password: "123".to_string(),
        });

        let res = user_logic.create_user(create_user_request).await;
        assert_eq!(res.unwrap_err().code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn create_user_duplicate_username_returns_error() {
        let mut mock_contexts = get_mock_contexts();
        let mut mock_services = get_mock_services();

        let password = "Password123".to_string();

        let user = user::Model {
            id: Default::default(),
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: password.clone(),
        };

        let create_user_request = Request::new(CreateUserRequest {
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: password.clone(),
        });

        mock_services
            .hashing_service_mock
            .expect_hash_password()
            .returning(move |_| Ok(password.clone()));

        mock_contexts
            .user_context_mock
            .expect_create()
            .with(predicate::eq(user.clone()))
            .returning(move |_| Err(DbErr::RecordNotInserted)); //todo!("Needs to be a SqlError with UniqueConstraintViolation with 'username' in message)

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let res = user_logic.create_user(create_user_request).await;
        assert_eq!(res.unwrap_err().code(), Code::Internal); //todo!("Needs to be code AlreadyExists when mocked Error is corrected)
    }

    #[tokio::test]
    async fn create_user_invalid_username_returns_error() {
        let mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let create_user_request = Request::new(CreateUserRequest {
            email: "valid@email.com".to_string(),
            username: "ØØØØØ".to_string(),
            password: "123".to_string(),
        });

        let res = user_logic.create_user(create_user_request).await;
        assert_eq!(res.unwrap_err().code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn create_user_valid_request_returns_ok() {
        let mut mock_contexts = get_mock_contexts();
        let mut mock_services = get_mock_services();

        let password = "Password123".to_string();

        let user = user::Model {
            id: Default::default(),
            email: "newuser@example.com".to_string(),
            username: "newuser".to_string(),
            password: password.clone(),
        };

        let create_user_request = Request::new(CreateUserRequest {
            email: "newuser@example.com".to_string(),
            username: "newuser".to_string(),
            password: password.clone(),
        });

        mock_services
            .hashing_service_mock
            .expect_hash_password()
            .returning(move |_| Ok(password.clone()));

        mock_contexts
            .user_context_mock
            .expect_create()
            .with(predicate::eq(user.clone()))
            .returning(move |_| Ok(user.clone()));

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let create_user_response = user_logic.create_user(create_user_request).await;
        assert!(create_user_response.is_ok());
    }

    #[tokio::test]
    async fn update_user_returns_ok() {
        let mut mock_contexts = get_mock_contexts();
        let mut mock_services = get_mock_services();

        let old_user = user::Model {
            id: 1,
            email: "olduser@example.com".to_string(),
            username: "old_username".to_string(),
            password: "StrongPassword123".to_string(),
        };

        let new_user = user::Model {
            id: 1,
            email: "newuser@example.com".to_string(),
            username: "new_username".to_string(),
            password: "g76df2gd7hd837g8hjd8723hd8gd823d82d3".to_string(),
        };

        mock_contexts
            .user_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| Ok(Some(old_user.clone())));

        mock_services
            .hashing_service_mock
            .expect_hash_password()
            .with(predicate::eq("StrongPassword123".to_string()))
            .returning(move |_| Ok("g76df2gd7hd837g8hjd8723hd8gd823d82d3".to_string()));

        mock_contexts
            .user_context_mock
            .expect_update()
            .with(predicate::eq(new_user.clone()))
            .returning(move |_| Ok(new_user.clone()));

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let mut update_user_request = Request::new(UpdateUserRequest {
            email: Some("newuser@example.com".to_string()),
            username: Some("new_username".to_string()),
            password: Some("StrongPassword123".to_string()),
        });

        update_user_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let update_user_response = user_logic.update_user(update_user_request).await;

        assert!(update_user_response.is_ok())
    }

    #[tokio::test]
    async fn update_user_non_existant_user_returns_err() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        mock_contexts
            .user_context_mock
            .expect_get_by_id()
            .with(predicate::eq(1))
            .returning(move |_| Err(DbErr::RecordNotFound("".to_string())));

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let mut update_user_request = Request::new(UpdateUserRequest {
            email: Some("new_test@test".to_string()),
            username: Some("new_test_user".to_string()),
            password: Some("new_test_pass".to_string()),
        });

        update_user_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let res = user_logic.update_user(update_user_request).await;

        assert_eq!(res.unwrap_err().code(), Code::Internal);
    }

    #[tokio::test]
    async fn get_users_returns_ok() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let users = vec![
            user::Model {
                id: 1,
                email: "".to_string(),
                username: "".to_string(),
                password: "".to_string(),
            },
            user::Model {
                id: 2,
                email: "".to_string(),
                username: "".to_string(),
                password: "".to_string(),
            },
        ];

        mock_contexts
            .user_context_mock
            .expect_get_by_ids()
            .returning(move |_| Ok(users.clone()));

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let get_users_request = Request::new(GetUsersRequest { ids: vec![1, 2] });

        let get_users_response = user_logic.get_users(get_users_request).await.unwrap();

        assert_eq!(get_users_response.get_ref().users.len(), 2);
    }

    #[tokio::test]
    async fn get_users_returns_empty_array() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let users: Vec<user::Model> = vec![];

        mock_contexts
            .user_context_mock
            .expect_get_by_ids()
            .returning(move |_| Ok(users.clone()));

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let user_logic = UserController::new(contexts, services);

        let get_users_request = Request::new(GetUsersRequest { ids: vec![1, 2] });

        let get_users_response = user_logic.get_users(get_users_request).await.unwrap();

        assert_eq!(get_users_response.get_ref().users.len(), 0);
    }
}
