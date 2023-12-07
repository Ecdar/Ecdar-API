use std::env;
use std::str::FromStr;

use crate::entities::{session, user};
use crate::tests::logics::helpers::{
    disguise_context_mocks, disguise_service_mocks, get_mock_contexts, get_mock_services,
};

use crate::api::auth::{Token, TokenType};
use crate::api::server::server::get_auth_token_request::{user_credentials, UserCredentials};
use crate::api::server::server::GetAuthTokenRequest;
use crate::logics::logic_impls::SessionLogic;
use crate::logics::logic_traits::SessionLogicTrait;
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
    let session_logic = SessionLogic::new(contexts, services);

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
    let session_logic = SessionLogic::new(contexts, services);

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
    let session_logic = SessionLogic::new(contexts, services);

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
    let session_logic = SessionLogic::new(contexts, services);

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
        .returning(move |_, _| true);

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
    let session_logic = SessionLogic::new(contexts, services);

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
        metadata::MetadataValue::from_str(format!("Bearer {}", refresh_token).as_str()).unwrap(),
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
    let session_logic = SessionLogic::new(contexts, services);

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
    let session_logic = SessionLogic::new(contexts, services);

    let response = session_logic.get_auth_token(request).await;

    assert_eq!(response.unwrap_err().code(), Code::Unauthenticated);
}
