use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode,
    errors::{Error, ErrorKind},
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use std::{env, str::FromStr};
use tonic::{metadata, Code, Request, Status};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    exp: usize,
}

pub enum TokenType {
    AccessToken,
    RefreshToken,
}

impl TokenType {
    /// An access token is valid for 20 minutes and a refresh token is valid for 90 days.
    pub fn duration(&self) -> i64 {
        match self {
            TokenType::AccessToken => Utc::now()
                .checked_add_signed(Duration::minutes(20))
                .expect("valid timestamp")
                .timestamp(),
            TokenType::RefreshToken => Utc::now()
                .checked_add_signed(Duration::days(90))
                .expect("valid timestamp")
                .timestamp(),
        }
    }

    pub fn secret(&self) -> String {
        match self {
            TokenType::AccessToken => env::var("ACCESS_TOKEN_HS512_SECRET")
                .expect("Expected ACCESS_TOKEN_HS512_SECRET to be set."),
            TokenType::RefreshToken => env::var("REFRESH_TOKEN_HS512_SECRET")
                .expect("Expected REFRESH_TOKEN_HS512_SECRET to be set."),
        }
    }
}

/// This method is used to create a new access or refresh token based on the token type and uid.
pub fn create_token(token_type: TokenType, uid: &str) -> Result<String, Error> {
    let claims = Claims {
        sub: uid.to_owned(),
        exp: token_type.duration() as usize,
    };

    let header = Header::new(Algorithm::HS512);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(token_type.secret().as_bytes()),
    )
        .map_err(|_| ErrorKind::InvalidToken.into())
}

/// This method is used to validate the access token (not refresh).
pub fn validation_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    let token = match get_token_from_request(&req) {
        Ok(token) => token,
        Err(err) => return Err(err),
    };

    match validate_token(token, TokenType::AccessToken) {
        Ok(token_data) => {
            req.metadata_mut().insert(
                "uid",
                metadata::MetadataValue::from_str(&token_data.claims.sub).unwrap(),
            );
            Ok(req)
        }
        Err(err) => Err(err),
    }
}

/// This method is used to get a token (access or refresh) from the request metadata.
pub fn get_token_from_request<T>(req: &Request<T>) -> Result<String, Status> {
    let token = match req.metadata().get("authorization") {
        Some(token) => token.to_str(),
        None => return Err(Status::unauthenticated("Token not found")),
    };

    match token {
        Ok(token) => Ok(token.trim_start_matches("Bearer ").to_string()),
        Err(_) => Err(Status::unauthenticated(
            "Could not read token from metadata",
        )),
    }
}

/// This method is used to validate a token (access or refresh).
/// It returns the token data if the token is valid.
pub fn validate_token(token: String, token_type: TokenType) -> Result<TokenData<Claims>, Status> {
    let mut validation = Validation::new(Algorithm::HS512);

    validation.validate_exp = true; // This might be redundant as this should be defualt, however, it doesn't seem to work without it.

    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(token_type.secret().as_bytes()),
        &validation,
    ) {
        Ok(c) => Ok(c),
        Err(err) => match *err.kind() {
            ErrorKind::InvalidToken => Err(Status::new(Code::Unauthenticated, "Token is invalid!")),
            ErrorKind::InvalidSignature => Err(Status::new(
                Code::Unauthenticated,
                "Token signature is invalid!",
            )),
            ErrorKind::ExpiredSignature => {
                Err(Status::new(Code::Unauthenticated, "Token is expired!"))
            }
            _ => Err(Status::new(Code::Internal, err.to_string())),
        },
    }
}

#[cfg(test)]
#[path = "../tests/api/auth.rs"]
mod tests;
