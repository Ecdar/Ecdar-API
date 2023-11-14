use chrono::Utc;
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

pub fn create_access_token(uid: &str) -> Result<String, Error> {
    let secret = env::var("ACCESS_TOKEN_HS512_SECRET")
        .expect("Expected ACCESS_TOKEN_HS512_SECRET to be set.");

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(20))
        .expect("valid timestamp")
        .timestamp();

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

pub fn create_refresh_token(uid: &str) -> Result<String, Error> {
    let secret = env::var("REFRESH_TOKEN_HS512_SECRET")
        .expect("Expected REFRESH_TOKEN_HS512_SECRET to be set.");

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(90))
        .expect("valid timestamp")
        .timestamp();

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

pub fn validate_token(token: String, is_refresh_token: bool) -> Result<TokenData<Claims>, Status> {
    let secret: String;

    if is_refresh_token {
        secret = env::var("REFRESH_TOKEN_HS512_SECRET").expect("Expected HS512_SECRET to be set.");
    } else {
        secret = env::var("ACCESS_TOKEN_HS512_SECRET").expect("Expected HS512_SECRET to be set.");
    }

    let mut validation = Validation::new(Algorithm::HS512);

    validation.validate_exp = true;

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
