use crate::api::auth::TokenType;
use crate::api::ecdar_api::update_session;
use crate::api::server::server::GetAuthTokenRequest;
use crate::entities::session;
use crate::tests::api::helpers::get_mock_services;
use mockall::predicate;
use sea_orm::DbErr;
use std::sync::Arc;
use tonic::{metadata, Code, Request};

#[tokio::test]
async fn update_session_no_session_exists_creates_session_returns_err() {
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
