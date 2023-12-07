use crate::api::auth::{RequestExt, Token, TokenError, TokenType};
use crate::api::context_collection::ContextCollection;
use crate::api::logic_traits::SessionLogicTrait;
use crate::api::server::server::get_auth_token_request::{user_credentials, UserCredentials};
use crate::api::server::server::{GetAuthTokenRequest, GetAuthTokenResponse};
use crate::database::context_traits::{SessionContextTrait, UserContextTrait};
use crate::entities::{session, user};
use sea_orm::DbErr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct SessionLogic {
    contexts: ContextCollection,
}

impl SessionLogic {
    pub fn new(contexts: ContextCollection) -> Self {
        Self { contexts }
    }

    /// Updates the session given by refresh token in the database.
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
            .unwrap();

        Ok((access_token, refresh_token))
    }
}

impl SessionLogicTrait for SessionLogic {
    async fn delete_session(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
    }

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

        let (access_token, refresh_token) = match message.user_credentials {
            None => {
                let refresh_token = Token::from_str(
                    TokenType::RefreshToken,
                    request
                        .token_str()
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
                let user = user_from_user_credentials(
                    self.contexts.user_context.clone(),
                    user_credentials,
                )
                .await
                .map_err(|err| Status::internal(err.to_string()))?
                .ok_or_else(|| Status::unauthenticated("Wrong username or password"))?;

                // Check if password in request matches users password
                if !self
                    .contexts
                    .hashing_context
                    .verify_password(input_password, user.password.as_str())
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
                        user_id: uid.parse().unwrap(),
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

async fn user_from_user_credentials(
    user_context: Arc<dyn UserContextTrait>,
    user_credentials: UserCredentials,
) -> Result<Option<user::Model>, DbErr> {
    match user_credentials.user {
        Some(user_credentials::User::Username(username)) => {
            Ok(user_context.get_by_username(username).await?)
        }
        Some(user_credentials::User::Email(email)) => Ok(user_context.get_by_email(email).await?),
        None => Ok(None),
    }
}

#[cfg(test)]
#[path = "../../tests/api/session_logic.rs"]
mod session_logic_tests;
