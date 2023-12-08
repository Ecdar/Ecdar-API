use crate::api::auth::{RequestExt, Token, TokenError, TokenType};
use crate::api::server::protobuf::get_auth_token_request::{user_credentials, UserCredentials};
use crate::api::server::protobuf::{GetAuthTokenRequest, GetAuthTokenResponse};
use crate::contexts::context_collection::ContextCollection;
use crate::controllers::controller_traits::SessionControllerTrait;
use crate::entities::{session, user};
use crate::services::service_collection::ServiceCollection;
use async_trait::async_trait;
use sea_orm::DbErr;
use tonic::{Code, Request, Response, Status};

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
        match user_credentials.user {
            Some(user_credentials::User::Username(username)) => {
                Ok(self.contexts.user_context.get_by_username(username).await?)
            }
            Some(user_credentials::User::Email(email)) => {
                Ok(self.contexts.user_context.get_by_email(email).await?)
            }
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
    /// Deletes the requester's session, found by their access token.
    ///  
    /// Returns the response that is received from Reveaal.
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

    /// This method is used to get a new access and refresh token for a user.
    ///
    /// # Errors
    /// This function will return an error if the user does not exist in the contexts,
    /// if the password in the request does not match the user's password,
    /// or if no user is provided in the request.
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
#[path = "../../tests/controllers/session_controller.rs"]
mod session_controller_tests;
