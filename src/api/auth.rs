use chrono::Utc;
use jsonwebtoken::{
    decode, encode,
    errors::{Error, ErrorKind},
    Algorithm, DecodingKey, EncodingKey, Header, Validation,
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
    let secret = env::var("ACCESS_TOKEN_HS512_SECRET").expect("Expected ACCESS_TOKEN_HS512_SECRET to be set.");

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
    let secret = env::var("REFRESH_TOKEN_HS512_SECRET").expect("Expected REFRESH_TOKEN_HS512_SECRET to be set.");

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

pub fn token_validation(mut req: Request<()>) -> Result<Request<()>, Status> {
    let token = match req.metadata().get("authorization") {
        Some(token) => token.to_str(),
        None => return Err(Status::unauthenticated("Token not found")),
    };

    let secret = env::var("HS512_SECRET").expect("Expected HS512_SECRET to be set.");

    let mut validation = Validation::new(Algorithm::HS512);

    validation.validate_exp = true;
    let token_data = match decode::<Claims>(
        token.unwrap().trim_start_matches("Bearer "),
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(c) => c,
        Err(err) => match *err.kind() {
            ErrorKind::InvalidToken => {
                return Err(Status::new(Code::Unauthenticated, "Token is invalid!"))
            }
            ErrorKind::InvalidSignature => {
                return Err(Status::new(
                    Code::Unauthenticated,
                    "Token signature is invalid!",
                ))
            }
            ErrorKind::ExpiredSignature => {
                return Err(Status::new(Code::Unauthenticated, "Token is expired!"))
            }
            _ => return Err(Status::new(Code::Internal, err.to_string())), // Example on how to handle a error generically
        },
    };

    req.metadata_mut().insert(
        "uid",
        metadata::MetadataValue::from_str(&token_data.claims.sub).unwrap(),
    );

    Ok(req)
}
