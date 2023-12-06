use crate::api::server::protobuf::GetAuthTokenRequest;
use crate::api::{auth::TokenType, ecdar_api::handle_session};
use crate::entities::session;
use crate::tests::api::helpers::get_mock_services;
use mockall::predicate;
use sea_orm::DbErr;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{metadata, Code, Request};

#[tokio::test]
async fn handle_session_updated_session_contains_correct_fields_returns_ok() {
    let mut mock_services = get_mock_services();

    let old_session = session::Model {
        id: 1,
        refresh_token: "old_refresh_token".to_string(),
        access_token: "old_access_token".to_string(),
        updated_at: Default::default(),
        user_id: 1,
    };

    let new_session = session::Model {
        id: 1,
        refresh_token: "new_refresh_token".to_string(),
        access_token: "new_access_token".to_string(),
        updated_at: Default::default(),
        user_id: 1,
    };

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::RefreshToken),
            predicate::eq("old_refresh_token".to_string()),
        )
        .returning(move |_, _| Ok(Some(old_session.clone())));

    mock_services
        .session_context_mock
        .expect_update()
        .with(predicate::eq(new_session.clone()))
        .returning(move |_| Ok(new_session.clone()));

    let mut get_auth_token_request = Request::new(GetAuthTokenRequest {
        user_credentials: None,
    });

    get_auth_token_request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer old_refresh_token").unwrap(),
    );

    let res = handle_session(
        Arc::new(mock_services.session_context_mock),
        &get_auth_token_request,
        false,
        "new_access_token".to_string(),
        "new_refresh_token".to_string(),
        "1".to_string(),
    )
    .await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn handle_session_no_session_exists_creates_session_returns_ok() {
    let mut mock_services = get_mock_services();

    let session = session::Model {
        id: Default::default(),
        refresh_token: "new_refresh_token".to_string(),
        access_token: "new_access_token".to_string(),
        updated_at: Default::default(),
        user_id: 1,
    };

    mock_services
        .session_context_mock
        .expect_create()
        .with(predicate::eq(session.clone()))
        .returning(move |_| Ok(session.clone()));

    let get_auth_token_request = Request::new(GetAuthTokenRequest {
        user_credentials: None,
    });

    let res = handle_session(
        Arc::new(mock_services.session_context_mock),
        &get_auth_token_request,
        true,
        "new_access_token".to_string(),
        "new_refresh_token".to_string(),
        "1".to_string(),
    )
    .await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn handle_session_no_session_exists_creates_session_returns_err() {
    let mut mock_services = get_mock_services();

    let session = session::Model {
        id: Default::default(),
        refresh_token: "new_refresh_token".to_string(),
        access_token: "new_access_token".to_string(),
        updated_at: Default::default(),
        user_id: 1,
    };

    mock_services
        .session_context_mock
        .expect_create()
        .with(predicate::eq(session.clone()))
        .returning(move |_| Err(DbErr::RecordNotInserted));

    let get_auth_token_request = Request::new(GetAuthTokenRequest {
        user_credentials: None,
    });

    let res = handle_session(
        Arc::new(mock_services.session_context_mock),
        &get_auth_token_request,
        true,
        "new_access_token".to_string(),
        "new_refresh_token".to_string(),
        "1".to_string(),
    )
    .await;

    assert_eq!(res.unwrap_err().code(), Code::Internal);
}
