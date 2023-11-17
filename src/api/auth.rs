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

/// This method is used to create a new access or refresh token based on the token type and uid.
/// An access token is valid for 20 minutes and a refresh token is valid for 90 days.
pub fn create_token(token_type: TokenType, uid: &str) -> Result<String, Error> {
    const ACCESS_TOKEN_DURATION_MINS: i64 = 20;
    const REFRESH_TOKEN_DURATION_DAYS: i64 = 90;

    let secret: String;
    let expiration: i64;

    match token_type {
        TokenType::AccessToken => {
            secret = env::var("ACCESS_TOKEN_HS512_SECRET")
                .expect("Expected ACCESS_TOKEN_HS512_SECRET to be set.");

            expiration = Utc::now()
                .checked_add_signed(Duration::minutes(ACCESS_TOKEN_DURATION_MINS))
                .expect("valid timestamp")
                .timestamp();
        }
        TokenType::RefreshToken => {
            secret = env::var("REFRESH_TOKEN_HS512_SECRET")
                .expect("Expected REFRESH_TOKEN_HS512_SECRET to be set.");

            expiration = Utc::now()
                .checked_add_signed(Duration::days(REFRESH_TOKEN_DURATION_DAYS))
                .expect("valid timestamp")
                .timestamp();
        }
    };

    let claims = Claims {
        sub: uid.to_owned(),
        exp: expiration as usize,
    };

    let header = Header::new(Algorithm::HS512);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| ErrorKind::InvalidToken.into())
}

/// This method is used to validate the access token (not refresh).
pub fn validation_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    let token = match get_token_from_request(&req) {
        Ok(token) => token,
        Err(err) => return Err(err),
    };

    match validate_token(token, false) {
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

    if token.is_ok() {
        Ok(token.unwrap().trim_start_matches("Bearer ").to_string())
    } else {
        Err(Status::unauthenticated(
            "Could not read token from metadata",
        ))
    }
}

/// This method is used to validate a token (access or refresh).
/// It returns the token data if the token is valid.
pub fn validate_token(token: String, is_refresh_token: bool) -> Result<TokenData<Claims>, Status> {
    let secret: String;

    if is_refresh_token {
        secret = env::var("REFRESH_TOKEN_HS512_SECRET").expect("Expected HS512_SECRET to be set.");
    } else {
        secret = env::var("ACCESS_TOKEN_HS512_SECRET").expect("Expected HS512_SECRET to be set.");
    }

    let mut validation = Validation::new(Algorithm::HS512);

    validation.validate_exp = true; // This might be redundant as this should be defualt, however, it doesn't seem to work without it.

    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
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
