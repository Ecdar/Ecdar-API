use crate::api::auth::{Token, TokenType};
use crate::api::server::server::ecdar_api_auth_server::EcdarApiAuth;
use crate::api::server::server::get_auth_token_request::{user_credentials, UserCredentials};
use crate::api::server::server::GetAuthTokenRequest;
use crate::entities::{session, user};
use crate::tests::api::helpers::{get_mock_concrete_ecdar_api, get_mock_services};
use std::env;
use std::str::FromStr;
use tonic::{metadata, Code, Request};

#[tokio::test]
async fn get_auth_token_from_credentials_returns_ok() {
    let mut mock_services = get_mock_services();

    let request = GetAuthTokenRequest {
        user_credentials: Option::from(UserCredentials {
            password: "Password123".to_string(),
            user: Option::from(user_credentials::User::Username("Example".to_string())),
        }),
    };

    mock_services
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
        .hashing_context_mock
        .expect_verify_password()
        .returning(move |_, _| true);

    mock_services
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

    let api = get_mock_concrete_ecdar_api(mock_services);
    let response = api.get_auth_token(Request::new(request)).await.unwrap();

    assert!(!response.get_ref().refresh_token.is_empty());
    assert!(!response.get_ref().access_token.is_empty());
}

#[tokio::test]
async fn get_auth_token_from_token_returns_ok() {
    env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

    let mut mock_services = get_mock_services();

    let mut request = Request::new(GetAuthTokenRequest {
        user_credentials: None,
    });

    let refresh_token = Token::new(TokenType::RefreshToken, "1").unwrap();

    request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str(format!("Bearer {}", refresh_token).as_str()).unwrap(),
    );

    mock_services
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

    mock_services
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

    let api = get_mock_concrete_ecdar_api(mock_services);

    let response = api.get_auth_token(request).await.unwrap();

    assert!(!response.get_ref().refresh_token.is_empty());
    assert!(!response.get_ref().access_token.is_empty());
}

#[tokio::test]
async fn get_auth_token_from_invalid_token_returns_err() {
    let mock_services = get_mock_services();

    let mut request = Request::new(GetAuthTokenRequest {
        user_credentials: None,
    });

    request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("invalid token").unwrap(),
    );

    let api = get_mock_concrete_ecdar_api(mock_services);

    let response = api.get_auth_token(request).await;

    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
}
