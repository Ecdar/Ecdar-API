use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};

use serde::{Deserialize, Serialize};
use std::{env, fmt::Display, str::FromStr};
use tonic::{metadata, Request, Status};

/// This method is used to validate the access token (not refresh).
pub fn validation_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    let token = match req.token_string() {
        Some(token) => Token::from_str(TokenType::AccessToken, &token),
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
/// use ecdar_api::api::auth::{Token, TokenType};
///
/// let token = Token::new(TokenType::AccessToken, "1").unwrap();
///
/// let token_data = token.validate().unwrap();
///
/// assert_eq!(token_data.claims.sub, "1");
/// assert_eq!(token.token_type(), TokenType::AccessToken);
/// assert_eq!(token.to_string(), token.as_str());
/// ```
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
    /// use ecdar_api::api::auth::{Token, TokenType};
    ///
    /// let token = Token::new(TokenType::AccessToken, "1").unwrap();
    /// ```
    pub fn new(token_type: TokenType, uid: &str) -> Result<Token, TokenError> {
        let now = Utc::now();
        let expiration = now
            .checked_add_signed(token_type.duration())
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: uid.to_owned(),
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
    /// Create a token from a string.
    ///
    /// # Arguments
    /// * `token_type` - The type of token to create.
    /// * `token` - The token string.
    ///
    /// # Examples
    /// ```
    /// use ecdar_api::api::auth::{Token, TokenType};
    ///
    /// let token = Token::from_str(TokenType::AccessToken, "token").unwrap();
    /// ```
    pub fn from_str(token_type: TokenType, token: &str) -> Token {
        Token {
            token_type,
            token: token.to_string(),
        }
    }
    /// Validate the token. Returns the token data if the token is valid.
    ///
    /// # Examples
    /// ```
    /// use ecdar_api::api::auth::{Token, TokenType};
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

    /// Returns the token as a string.
    // pub fn to_string(&self) -> String {
    //     self.token.clone()
    // }
    /// Extracts the token as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use ecdar_api::api::auth::{Token, TokenType};
    ///
    /// let token = Token::new(TokenType::AccessToken, "1").unwrap();
    ///
    /// assert_eq!(token.as_str(), "token");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.token
    }
    /// Returns the token type.
    ///
    /// # Examples
    ///
    /// ```
    /// use ecdar_api::api::auth::{Token, TokenType};
    ///
    /// let token = Token::new(TokenType::AccessToken, "1").unwrap();
    ///
    /// assert_eq!(token.token_type(), TokenType::AccessToken);
    /// ```
    pub fn token_type(&self) -> TokenType {
        self.token_type.clone()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

#[derive(Debug)]
pub enum TokenError {
    InvalidToken,
    InvalidSignature,
    ExpiredSignature,
    Custom(String),
}

/// This is used to get the token error as a string.
impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::InvalidToken => write!(f, "Invalid token"),
            TokenError::InvalidSignature => write!(f, "Invalid signature"),
            TokenError::ExpiredSignature => write!(f, "Expired signature"),
            TokenError::Custom(message) => write!(f, "{}", message),
        }
    }
}

/// This is used to convert the jsonwebtoken error kind to a [TokenError].
impl From<jsonwebtoken::errors::ErrorKind> for TokenError {
    fn from(error: jsonwebtoken::errors::ErrorKind) -> Self {
        match error {
            jsonwebtoken::errors::ErrorKind::InvalidToken => TokenError::InvalidToken,
            jsonwebtoken::errors::ErrorKind::InvalidSignature => TokenError::InvalidSignature,
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::ExpiredSignature,
            _ => TokenError::Custom("Failed to validate token".to_string()),
        }
    }
}

/// This is used to convert the jsonwebtoken error to a [TokenError].
impl From<jsonwebtoken::errors::Error> for TokenError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        TokenError::from(error.kind().clone())
    }
}

/// This is used to convert the [TokenError] to a [Status].
impl From<TokenError> for Status {
    fn from(error: TokenError) -> Self {
        Status::unauthenticated(error.to_string())
    }
}

/// An extension trait for [Request]`s that provides a variety of convenient
/// auth related methods.
pub trait RequestExt {
    fn token_string(&self) -> Option<String>;
    fn token_str(&self) -> Option<&str>;

    fn uid(&self) -> Option<i32>;
}

impl<T> RequestExt for Request<T> {
    /// Returns the token string from the request metadata.
    fn token_string(&self) -> Option<String> {
        self.metadata().get("authorization").map(|token| {
            token
                .to_str()
                .expect("failed to parse token string") //TODO better error handling
                .trim_start_matches("Bearer ")
                .to_string()
        })
    }
    /// Returns the token string slice from the request metadata.
    fn token_str(&self) -> Option<&str> {
        match self.metadata().get("authorization") {
            //TODO better error handling
            Some(token) => Some(
                token
                    .to_str()
                    .expect("failed to parse token string")
                    .trim_start_matches("Bearer "),
            ),
            None => None,
        }
    }

    /// Returns the uid from the request metadata.
    fn uid(&self) -> Option<i32> {
        //TODO better error handling
        let uid = match self
            .metadata()
            .get("uid")
            .expect("failed to parse user id")
            .to_str()
        {
            Ok(uid) => uid,
            Err(_) => return None,
        };
        //TODO better error handling
        Some(uid.parse().expect("failed to parse user id"))
    }
}

#[cfg(test)]
#[path = "../tests/api/auth.rs"]
mod tests;
