use crate::api::auth::{RequestExt, Token, TokenError, TokenType};
use crate::api::server::protobuf::get_auth_token_request::user_credentials::User;
use crate::api::server::protobuf::{
    get_auth_token_request::UserCredentials, GetAuthTokenRequest, GetAuthTokenResponse,
};
use crate::contexts::ContextCollection;
use crate::entities::{session, user};
use crate::services::ServiceCollection;
use async_trait::async_trait;
use sea_orm::DbErr;
use tonic::{Request, Response, Status};

#[async_trait]
pub trait SessionControllerTrait: Send + Sync {
    /// Deletes the requester's session, found by their access token.
    ///
    /// Returns the response that is received from Reveaal.
    async fn delete_session(&self, _request: Request<()>) -> Result<Response<()>, Status>;

    /// This method is used to get a new access and refresh token for a user.
    ///
    /// # Errors
    /// This function will return an error if the user does not exist in the contexts,
    /// if the password in the request does not match the user's password,
    /// or if no user is provided in the request.
    async fn get_auth_token(
        &self,
        request: Request<GetAuthTokenRequest>,
    ) -> Result<Response<GetAuthTokenResponse>, Status>;
}

pub struct SessionController {
    contexts: ContextCollection,
    services: ServiceCollection,
}

impl SessionController {
    pub fn new(contexts: ContextCollection, services: ServiceCollection) -> Self {
        Self { contexts, services }
    }

    async fn user_from_user_credentials(
        &self,
        user_credentials: UserCredentials,
    ) -> Result<Option<user::Model>, DbErr> {
        let m = user_credentials.user.map(|u| match u {
            User::Username(username) => self.contexts.user_context.get_by_username(username),
            User::Email(email) => self.contexts.user_context.get_by_email(email),
        });
        match m {
            Some(l) => Ok(l.await?),
            None => Ok(None),
        }
    }

    /// Updates the session given by refresh token in the contexts.
    /// Returns the new access and refresh token i.e. a tuple `(Token, Token)` where the 0th element is the access token and the 1st element refresh token.
    pub async fn update_session(&self, refresh_token: String) -> Result<(Token, Token), Status> {
        let session = match self
            .contexts
            .session_context
            .get_by_token(TokenType::RefreshToken, refresh_token)
            .await
        {
            Ok(Some(session)) => session,
            Ok(None) => {
                return Err(Status::unauthenticated(
                    "No session found with given refresh token",
                ));
            }
            Err(err) => return Err(Status::internal(err.to_string())),
        };

        let uid = session.user_id.to_string();

        let access_token = Token::access(&uid)?;
        let refresh_token = Token::refresh(&uid)?;

        self.contexts
            .session_context
            .update(session::Model {
                id: session.id,
                access_token: access_token.to_string(),
                refresh_token: refresh_token.to_string(),
                updated_at: Default::default(),
                user_id: session.user_id,
            })
            .await
            .map_err(|err| {
                Status::internal(format!(
                    "a database error occurred, internal message: {}",
                    err
                ))
            })?;

        Ok((access_token, refresh_token))
    }
}

#[async_trait]
impl SessionControllerTrait for SessionController {
    async fn delete_session(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let access_token = request
            .token_string()
            .map_err(|err| {
                Status::internal(format!(
                    "failed to convert token to string, internal error: {}",
                    err
                ))
            })?
            .ok_or(Status::unauthenticated("No access token provided"))?;

        self.contexts
            .session_context
            .delete_by_token(TokenType::AccessToken, access_token)
            .await
            .map(|_| Response::new(()))
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn get_auth_token(
        &self,
        request: Request<GetAuthTokenRequest>,
    ) -> Result<Response<GetAuthTokenResponse>, Status> {
        let message = request.get_ref().clone();

        let (access_token, refresh_token) = match message.user_credentials {
            None => {
                let refresh_token = Token::from_str(
                    TokenType::RefreshToken,
                    request
                        .token_str()
                        .map_err(|err| {
                            Status::internal(format!(
                                "failed to convert token to string, internal error: {}",
                                err
                            ))
                        })?
                        .ok_or(Status::unauthenticated("No refresh token provided"))?,
                );

                // Validate refresh token
                match refresh_token.validate() {
                    Ok(_) => (),
                    Err(TokenError::ExpiredSignature) => {
                        // Delete session if expired
                        let _ = self
                            .contexts
                            .session_context
                            .delete_by_token(TokenType::RefreshToken, refresh_token.to_string())
                            .await;

                        return Err(Status::from(TokenError::ExpiredSignature));
                    }
                    Err(err) => return Err(Status::from(err)),
                }

                self.update_session(refresh_token.to_string()).await?
            }
            Some(user_credentials) => {
                let input_password = user_credentials.password.clone();
                let user = self
                    .user_from_user_credentials(user_credentials)
                    .await
                    .map_err(|err| Status::internal(err.to_string()))?
                    .ok_or_else(|| Status::unauthenticated("Wrong username or password"))?;

                // Check if password in request matches users password
                if !self
                    .services
                    .hashing_service
                    .verify_password(input_password, user.password.as_str())
                    .map_err(|__err| Status::internal("failed to verify password"))?
                {
                    return Err(Status::unauthenticated("Wrong username or password"));
                }

                let uid = user.id.to_string();

                let access_token = Token::access(&uid)?;
                let refresh_token = Token::refresh(&uid)?;

                self.contexts
                    .session_context
                    .create(session::Model {
                        id: Default::default(),
                        access_token: access_token.to_string(),
                        refresh_token: refresh_token.to_string(),
                        updated_at: Default::default(),
                        user_id: uid.parse().map_err(|err| {
                            Status::internal(format!(
                                "failed to parse user id, internal error: {}",
                                err
                            ))
                        })?,
                    })
                    .await
                    .map_err(|err| Status::internal(err.to_string()))?;

                (access_token, refresh_token)
            }
        };

        Ok(Response::new(GetAuthTokenResponse {
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate;
    use std::env;
    use std::str::FromStr;

    use super::super::helpers::{
        disguise_context_mocks, disguise_service_mocks, get_mock_contexts, get_mock_services,
    };
    use crate::entities::{session, user};

    use crate::api::auth::{Token, TokenType};
    use crate::api::server::protobuf::get_auth_token_request::{user_credentials, UserCredentials};
    use crate::api::server::protobuf::GetAuthTokenRequest;
    use crate::controllers::SessionController;
    use crate::controllers::SessionControllerTrait;
    use sea_orm::DbErr;
    use tonic::{metadata, Code, Request};

    #[tokio::test]
    async fn update_session_no_session_exists_creates_session_returns_err() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .returning(move |_, _| Ok(None));

        mock_contexts
            .session_context_mock
            .expect_update()
            .returning(move |_| Err(DbErr::RecordNotInserted));

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let session_logic = SessionController::new(contexts, services);

        let res = session_logic
            .update_session("old_refresh_token".to_string())
            .await;

        assert_eq!(res.unwrap_err().code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn update_session_returns_new_tokens_when_session_exists() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let refresh_token = "refresh_token".to_string();

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .times(1)
            .returning(|_, _| {
                Ok(Some(session::Model {
                    id: 0,
                    access_token: "old_access_token".to_string(),
                    refresh_token: "old_refresh_token".to_string(),
                    updated_at: Default::default(),
                    user_id: 1,
                }))
            });

        mock_contexts
            .session_context_mock
            .expect_update()
            .times(1)
            .returning(move |_| {
                Ok(session::Model {
                    id: 0,
                    refresh_token: "refresh_token".to_string(),
                    access_token: "access_token".to_string(),
                    updated_at: Default::default(),
                    user_id: 1,
                })
            });

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let session_logic = SessionController::new(contexts, services);

        let result = session_logic.update_session(refresh_token).await;

        assert!(result.is_ok());
        let (access_token, refresh_token) = result.unwrap();
        assert_ne!(access_token.to_string(), "old_access_token");
        assert_ne!(refresh_token.to_string(), "old_refresh_token");
    }

    #[tokio::test]
    async fn update_session_returns_error_when_no_session_found() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let refresh_token = "refresh_token".to_string();

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .times(1)
            .returning(|_, _| Ok(None));

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let session_logic = SessionController::new(contexts, services);

        let result = session_logic.update_session(refresh_token).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn update_session_returns_error_when_database_error_occurs() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let refresh_token = "refresh_token".to_string();

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .times(1)
            .returning(|_, _| Err(DbErr::RecordNotFound("".to_string())));

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let session_logic = SessionController::new(contexts, services);

        let result = session_logic.update_session(refresh_token).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), Code::Internal);
    }

    #[tokio::test]
    async fn get_auth_token_from_credentials_returns_ok() {
        let mut mock_contexts = get_mock_contexts();
        let mut mock_services = get_mock_services();

        let request = GetAuthTokenRequest {
            user_credentials: Option::from(UserCredentials {
                password: "Password123".to_string(),
                user: Option::from(user_credentials::User::Username("Example".to_string())),
            }),
        };

        mock_contexts
            .user_context_mock
            .expect_get_by_username()
            .returning(move |_| {
                Ok(Option::from(user::Model {
                    id: 1,
                    email: "".to_string(),
                    username: "Example".to_string(),
                    password: "".to_string(),
                }))
            });

        mock_services
            .hashing_service_mock
            .expect_verify_password()
            .returning(move |_, _| Ok(true));

        mock_contexts
            .session_context_mock
            .expect_create()
            .returning(move |_| {
                Ok(session::Model {
                    id: 0,
                    refresh_token: "refresh_token".to_string(),
                    access_token: "access_token".to_string(),
                    updated_at: Default::default(),
                    user_id: 1,
                })
            });

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let session_logic = SessionController::new(contexts, services);

        let response = session_logic
            .get_auth_token(Request::new(request))
            .await
            .unwrap();

        assert!(!response.get_ref().refresh_token.is_empty());
        assert!(!response.get_ref().access_token.is_empty());
    }

    #[tokio::test]
    async fn get_auth_token_from_token_returns_ok() {
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let mut request = Request::new(GetAuthTokenRequest {
            user_credentials: None,
        });

        let refresh_token = Token::new(TokenType::RefreshToken, "1").unwrap();

        request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str(format!("Bearer {}", refresh_token).as_str())
                .unwrap(),
        );

        mock_contexts
            .session_context_mock
            .expect_get_by_token()
            .returning(move |_, _| {
                Ok(Option::from(session::Model {
                    id: 0,
                    refresh_token: "refresh_token".to_string(),
                    access_token: "access_token".to_string(),
                    updated_at: Default::default(),
                    user_id: 1,
                }))
            });

        mock_contexts
            .session_context_mock
            .expect_update()
            .returning(move |_| {
                Ok(session::Model {
                    id: 0,
                    refresh_token: "refresh_token".to_string(),
                    access_token: "access_token".to_string(),
                    updated_at: Default::default(),
                    user_id: 1,
                })
            });

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let session_logic = SessionController::new(contexts, services);

        let response = session_logic.get_auth_token(request).await.unwrap();

        assert!(!response.get_ref().refresh_token.is_empty());
        assert!(!response.get_ref().access_token.is_empty());
    }

    #[tokio::test]
    async fn get_auth_token_from_invalid_token_returns_err() {
        let mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        let mut request = Request::new(GetAuthTokenRequest {
            user_credentials: None,
        });

        request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str("invalid token").unwrap(),
        );

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let session_logic = SessionController::new(contexts, services);

        let response = session_logic.get_auth_token(request).await;

        assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn delete_session_returns_ok() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        mock_contexts
            .session_context_mock
            .expect_delete_by_token()
            .with(
                predicate::eq(TokenType::AccessToken),
                predicate::eq("test_token".to_string()),
            )
            .returning(move |_, _| {
                Ok(session::Model {
                    id: 1,
                    refresh_token: Default::default(),
                    access_token: "test_token".to_string(),
                    updated_at: Default::default(),
                    user_id: Default::default(),
                })
            });

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let session_logic = SessionController::new(contexts, services);

        let mut request = Request::new(());
        request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str("Bearer test_token").unwrap(),
        );

        let res = session_logic.delete_session(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn delete_session_no_session_returns_err() {
        let mut mock_contexts = get_mock_contexts();
        let mock_services = get_mock_services();

        mock_contexts
            .session_context_mock
            .expect_delete_by_token()
            .with(
                predicate::eq(TokenType::AccessToken),
                predicate::eq("test_token".to_string()),
            )
            .returning(move |_, _| {
                Err(DbErr::RecordNotFound(
                    "No session found with the provided access token".to_string(),
                ))
            });

        let contexts = disguise_context_mocks(mock_contexts);
        let services = disguise_service_mocks(mock_services);
        let session_logic = SessionController::new(contexts, services);

        let mut request = Request::new(());
        request.metadata_mut().insert(
            "authorization",
            metadata::MetadataValue::from_str("Bearer test_token").unwrap(),
        );

        let res = session_logic.delete_session(request).await;

        assert_eq!(res.unwrap_err().code(), Code::Internal);
    }
}
