use crate::api::ecdar_api::update_session;

use crate::entities::session;
use crate::tests::api::helpers::get_mock_services;

use sea_orm::DbErr;
use std::sync::Arc;
use tonic::Code;

#[tokio::test]
async fn update_session_no_session_exists_creates_session_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .returning(move |_, _| Ok(None));

    mock_services
        .session_context_mock
        .expect_update()
        .returning(move |_| Err(DbErr::RecordNotInserted));

    let res = update_session(
        Arc::new(mock_services.session_context_mock),
        "old_refresh_token".to_string(),
    )
    .await;

    assert_eq!(res.unwrap_err().code(), Code::Unauthenticated);
}

#[tokio::test]
async fn update_session_returns_new_tokens_when_session_exists() {
    let mut mock_services = get_mock_services();
    let refresh_token = "refresh_token".to_string();

    mock_services
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

    mock_services
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

    let result = update_session(Arc::new(mock_services.session_context_mock), refresh_token).await;

    assert!(result.is_ok());
    let (access_token, refresh_token) = result.unwrap();
    assert_ne!(access_token.to_string(), "old_access_token");
    assert_ne!(refresh_token.to_string(), "old_refresh_token");
}

#[tokio::test]
async fn update_session_returns_error_when_no_session_found() {
    let mut mock_services = get_mock_services();
    let refresh_token = "refresh_token".to_string();

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .times(1)
        .returning(|_, _| Ok(None));

    let result = update_session(Arc::new(mock_services.session_context_mock), refresh_token).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), Code::Unauthenticated);
}

#[tokio::test]
async fn update_session_returns_error_when_database_error_occurs() {
    let mut mock_services = get_mock_services();
    let refresh_token = "refresh_token".to_string();

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .times(1)
        .returning(|_, _| Err(DbErr::RecordNotFound("".to_string())));

    let result = update_session(Arc::new(mock_services.session_context_mock), refresh_token).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), Code::Internal);
}
