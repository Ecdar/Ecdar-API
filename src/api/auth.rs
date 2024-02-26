use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};

use jsonwebtoken::errors::{Error, ErrorKind};
use serde::{Deserialize, Serialize};
use std::{env, fmt::Display, str::FromStr};
use tonic::{
    metadata::{self, errors::ToStrError},
    Request, Status,
};

/// This method is used to validate the access token (not refresh).
pub fn validation_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    /*
    let token = match req.token_string().map_err(|err|
        Status::internal(format!(
            "could not stringify user id in request metadata, internal error {}",
            err
        ))
     */
    let token = match req
        .token_string()
        .map_err(|e| {
            format!(
                "could not stringify user id in request metadata, internal error {}",
                e
            )
        })
        .map_err(Status::internal)?
    {
        Some(token) => Token::from_str(TokenType::AccessToken, token),
        None => return Err(Status::unauthenticated("Token not found")),
    };

    match token.validate() {
        Ok(token_data) => {
            req.metadata_mut().insert(
                "uid",
                metadata::MetadataValue::from_str(&token_data.claims.sub)
                    .map_err(|err| Status::internal(err.to_string()))?,
            );
            Ok(req)
        }
        Err(err) => Err(err.into()),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    exp: usize,
}

/// Enumerator for specifying the token type.
#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    AccessToken,
    RefreshToken,
}

impl TokenType {
    /// Get the duration for the token type.
    fn duration(&self) -> Duration {
        match self {
            TokenType::AccessToken => Duration::minutes(20),
            TokenType::RefreshToken => Duration::days(90),
        }
    }
    /// Get the secret for the token type.
    ///
    /// # Panics
    /// This method will panic if the token secret environment variable is not set.
    fn secret(&self) -> String {
        match self {
            TokenType::AccessToken => env::var("ACCESS_TOKEN_HS512_SECRET")
                .expect("env variable `ACCESS_TOKEN_HS512_SECRET` is not set"),
            TokenType::RefreshToken => env::var("REFRESH_TOKEN_HS512_SECRET")
                .expect("env variable `REFRESH_TOKEN_HS512_SECRET` is not set"),
        }
    }
}

/// This struct is used to create, validate and extract a token.
///
/// # Examples
///
/// ```
/// use ecdar_api::controllers::auth::{Token, TokenType};
///
/// let token = Token::new(TokenType::AccessToken, "1").unwrap();
///
/// let token_data = token.validate().unwrap();
///
/// assert_eq!(token_data.claims.sub, "1");
/// assert_eq!(token.token_type(), TokenType::AccessToken);
/// assert_eq!(token.to_string(), token.as_str());
/// ```
#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    token: String,
}

impl Token {
    /// Creates a new Json Web Token.
    ///
    /// # Arguments
    /// * `token_type` - The type of token to create.
    /// * `uid` - The user id to create the token for.
    ///
    /// # Examples
    /// ```
    /// use ecdar_api::controllers::auth::{Token, TokenType};
    ///
    /// let token = Token::new(TokenType::AccessToken, "1").unwrap();
    /// ```
    pub fn new<T: Into<String>>(token_type: TokenType, uid: T) -> Result<Token, TokenError> {
        let now = Utc::now();
        let expiration = now
            .checked_add_signed(token_type.duration())
            .ok_or(TokenError::InvalidSignature)?
            .timestamp();

        let claims = Claims {
            sub: uid.into(),
            exp: expiration as usize,
        };

        let header = Header::new(Algorithm::HS512);

        let token = encode(
            &header,
            &claims,
            &EncodingKey::from_secret(token_type.secret().as_bytes()),
        )?;

        Ok(Token { token_type, token })
    }

    /// Creates a new refresh token.
    ///
    /// # Arguments
    /// * `uid` - The user id to create the token for.
    ///
    /// # Examples
    /// ```
    /// use ecdar_api::controllers::auth::{Token, TokenType};
    ///
    /// let refresh_token = Token::refresh("1").unwrap();
    ///
    /// assert_eq!(refresh_token.token_type(), TokenType::RefreshToken);
    /// ```
    pub fn refresh<T: Into<String>>(uid: T) -> Result<Token, TokenError> {
        Token::new(TokenType::RefreshToken, uid)
    }

    /// Creates a new access token.
    ///
    /// # Arguments
    /// * `uid` - The user id to create the token for.
    ///
    /// # Examples
    /// ```
    /// use ecdar_api::controllers::auth::{Token, TokenType};
    ///
    /// let access_token = Token::access("1").unwrap();
    ///
    /// assert_eq!(access_token.token_type(), TokenType::AccessToken);
    /// ```
    pub fn access<T: Into<String>>(uid: T) -> Result<Token, TokenError> {
        Token::new(TokenType::AccessToken, uid)
    }

    /// Create a token from a string.
    ///
    /// # Arguments
    /// * `token_type` - The type of token to create.
    /// * `token` - The token string.
    ///
    /// # Examples
    /// ```
    /// use ecdar_api::controllers::auth::{Token, TokenType};
    ///
    /// let token = Token::from_str(TokenType::AccessToken, "token")
    /// ```
    pub fn from_str<T: Into<String>>(token_type: TokenType, token: T) -> Token {
        Token {
            token_type,
            token: token.into(),
        }
    }
    /// Validate the token. Returns the token data if the token is valid.
    ///
    /// # Examples
    /// ```
    /// use ecdar_api::controllers::auth::{Token, TokenType};
    ///
    /// let token = Token::new(TokenType::AccessToken, "1").unwrap();
    /// let token_data = token.validate().unwrap();
    ///
    /// assert_eq!(token_data.claims.sub, "1");
    /// ```
    pub fn validate(&self) -> Result<TokenData<Claims>, TokenError> {
        let mut validation = Validation::new(Algorithm::HS512);

        validation.validate_exp = true; // This might be redundant as this should be default, however, it doesn't seem to work without it.

        match decode::<Claims>(
            &self.token,
            &DecodingKey::from_secret(self.token_type.secret().as_bytes()),
            &validation,
        ) {
            Ok(c) => Ok(c),
            Err(err) => Err(err.into()),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

/// Token errors that can be returned by authentication.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TokenError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Expired signature")]
    ExpiredSignature,
    #[error("{0}")]
    Unknown(String),
}

/// This is used to convert a [ErrorKind] to a [TokenError].
impl From<ErrorKind> for TokenError {
    fn from(error_kind: ErrorKind) -> Self {
        match error_kind {
            ErrorKind::InvalidToken => TokenError::InvalidToken,
            ErrorKind::InvalidSignature => TokenError::InvalidSignature,
            ErrorKind::ExpiredSignature => TokenError::ExpiredSignature,
            _ => TokenError::Unknown("Unknown token error".to_string()),
        }
    }
}

/// This is used to convert a [Error] to a [TokenError].
impl From<Error> for TokenError {
    fn from(error: Error) -> Self {
        TokenError::from(error.kind().clone())
    }
}

/// This is used to convert a [TokenError] to a [Status].
impl From<TokenError> for Status {
    fn from(error: TokenError) -> Self {
        Status::unauthenticated(error.to_string())
    }
}

/// An extension trait for [Request]`s that provides a variety of convenient
/// auth related methods.
pub trait RequestExt {
    fn token_str(&self) -> Result<Option<&str>, ToStrError>;
    fn token_string(&self) -> Result<Option<String>, ToStrError>;
    fn uid(&self) -> Result<Option<i32>, ToStrError>;
}

impl<T> RequestExt for Request<T> {
    /// Returns the token string slice from the request metadata.
    fn token_str(&self) -> Result<Option<&str>, ToStrError> {
        match self.metadata().get("authorization") {
            Some(token) => Ok(Some(token.to_str()?.trim_start_matches("Bearer "))),
            None => Ok(None),
        }
    }

    /// Returns the token string from the request metadata.
    fn token_string(&self) -> Result<Option<String>, ToStrError> {
        let res = self.metadata().get("authorization");
        match res {
            Some(val) => Ok(Some(
                val.to_str()?.trim_start_matches("Bearer ").to_string(),
            )),
            None => Ok(None),
        }
    }
    /// Returns the uid from the request metadata.
    fn uid(&self) -> Result<Option<i32>, ToStrError> {
        match self.metadata().get("uid") {
            Some(val) => match val.to_str()?.parse::<i32>() {
                Ok(val) => Ok(Some(val)),
                Err(_err) => Ok(None),
            },
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::auth::{RequestExt, Token, TokenError, TokenType};
    use std::{env, str::FromStr};
    use tonic::{metadata::MetadataValue, Request};

    #[tokio::test]
    async fn request_token_trims_bearer() {
        let token = "Bearer 1234567890";
        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert("authorization", MetadataValue::from_str(token).unwrap());

        let result = request.token_str().unwrap().unwrap();

        assert_eq!(result, token.trim_start_matches("Bearer "));
    }

    #[tokio::test]
    async fn request_token_no_token_returns_none() {
        let request = Request::new(());
        let result = request.token_str().unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn token_new_access_returns_token() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let uid = "1";
        let result = Token::new(TokenType::AccessToken, uid);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn token_new_refresh_returns_token() {
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let uid = "1";
        let result = Token::new(TokenType::RefreshToken, uid);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_token_valid_access_returns_tokendata() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = Token::new(TokenType::AccessToken, "1").unwrap();
        let result = token.validate();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_token_valid_refresh_returns_tokendata() {
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let token = Token::new(TokenType::RefreshToken, "1").unwrap();
        let result = token.validate();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_token_invalid_returns_err() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let result_access = Token::from_str(TokenType::AccessToken, "invalid_token").validate();
        let result_refresh = Token::from_str(TokenType::RefreshToken, "invalid_token").validate();

        assert_eq!(result_access.unwrap_err(), TokenError::InvalidToken);
        assert_eq!(result_refresh.unwrap_err(), TokenError::InvalidToken);
    }

    #[tokio::test]
    async fn token_type_access_returns_access() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = Token::new(TokenType::AccessToken, "1").unwrap();
        let result = token.token_type;

        assert_eq!(result, TokenType::AccessToken);
    }

    #[tokio::test]
    async fn token_type_refresh_returns_refresh() {
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let token = Token::new(TokenType::RefreshToken, "1").unwrap();
        let result = token.token_type;

        assert_eq!(result, TokenType::RefreshToken);
    }

    #[tokio::test]
    async fn token_to_string_returns_string() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = Token::new(TokenType::AccessToken, "1").unwrap();
        let result = token.to_string();

        assert_eq!(result, token.token);
    }

    #[tokio::test]
    async fn token_from_str_returns_token() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = Token::new(TokenType::AccessToken, "1").unwrap();
        let token_from_str = Token::from_str(TokenType::AccessToken, token.token);

        let result = token_from_str.validate();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn token_from_str_invalid_returns_err() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = Token::from_str(TokenType::AccessToken, "invalid_token");
        let result = token.validate();

        assert!(result.is_err());
    }
}
