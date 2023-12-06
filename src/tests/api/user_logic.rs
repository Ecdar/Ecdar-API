use crate::api::server::protobuf::ecdar_api_auth_server::EcdarApiAuth;
use crate::api::server::protobuf::ecdar_api_server::EcdarApi;
use crate::api::server::protobuf::{CreateUserRequest, UpdateUserRequest};
use crate::entities::user;
use crate::tests::api::helpers::{get_mock_concrete_ecdar_api, get_mock_services};
use mockall::predicate;
use sea_orm::DbErr;
use std::str::FromStr;
use tonic::{metadata, Code, Request};

#[tokio::test]
async fn delete_user_nonexistent_user_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .user_context_mock
        .expect_delete()
        .with(predicate::eq(1))
        .returning(|_| Err(DbErr::RecordNotFound("".into())));

    let api = get_mock_concrete_ecdar_api(mock_services);

    let mut delete_request = Request::new(());

    // Insert uid into request metadata
    delete_request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let delete_response = api.delete_user(delete_request).await.unwrap_err();
    let expected_response_code = Code::Internal;

    assert_eq!(delete_response.code(), expected_response_code);
}

#[tokio::test]
async fn delete_user_existing_user_returns_ok() {
    let mut mock_services = get_mock_services();

    let user = user::Model {
        id: 1,
        email: "".to_string(),
        username: "".to_string(),
        password: "".to_string(),
    };

    mock_services
        .user_context_mock
        .expect_delete()
        .with(predicate::eq(1))
        .returning(move |_| Ok(user.clone()));

    let api = get_mock_concrete_ecdar_api(mock_services);

    let mut delete_request = Request::new(());

    // Insert uid into request metadata
    delete_request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let delete_response = api.delete_user(delete_request).await;

    assert!(delete_response.is_ok());
}

#[tokio::test]
async fn create_user_nonexistent_user_returns_ok() {
    let mut mock_services = get_mock_services();

    let user = user::Model {
        id: Default::default(),
        email: "anders21@student.aau.dk".to_string(),
        username: "anders".to_string(),
        password: "123".to_string(),
    };

    mock_services
        .user_context_mock
        .expect_create()
        .with(predicate::eq(user.clone()))
        .returning(move |_| Ok(user.clone()));

    let api = get_mock_concrete_ecdar_api(mock_services);

    let create_user_request = Request::new(CreateUserRequest {
        email: "anders21@student.aau.dk".to_string(),
        username: "anders".to_string(),
        password: "123".to_string(),
    });

    let create_user_response = api.create_user(create_user_request).await;
    assert!(create_user_response.is_ok());
}

#[tokio::test]
async fn create_user_duplicate_email_returns_error() {
    let mut mock_services = get_mock_services();

    let user = user::Model {
        id: Default::default(),
        email: "anders21@student.aau.dk".to_string(),
        username: "anders".to_string(),
        password: "".to_string(),
    };

    mock_services
        .user_context_mock
        .expect_create()
        .with(predicate::eq(user.clone()))
        .returning(move |_| Err(DbErr::RecordNotInserted)); //todo!("Needs to be a SqlError with UniqueConstraintViolation with 'email' in message)

    let api = get_mock_concrete_ecdar_api(mock_services);

    let create_user_request = Request::new(CreateUserRequest {
        email: "anders21@student.aau.dk".to_string(),
        username: "anders".to_string(),
        password: "".to_string(),
    });

    let res = api.create_user(create_user_request).await;
    assert_eq!(res.unwrap_err().code(), Code::Internal); //todo!("Needs to be code AlreadyExists when mocked Error is corrected)
}

#[tokio::test]
async fn create_user_invalid_email_returns_error() {
    let mock_services = get_mock_services();

    let api = get_mock_concrete_ecdar_api(mock_services);

    let create_user_request = Request::new(CreateUserRequest {
        email: "invalid-email".to_string(),
        username: "newuser".to_string(),
        password: "123".to_string(),
    });

    let res = api.create_user(create_user_request).await;
    assert_eq!(res.unwrap_err().code(), Code::InvalidArgument);
}

#[tokio::test]
async fn create_user_duplicate_username_returns_error() {
    let mut mock_services = get_mock_services();

    let user = user::Model {
        id: Default::default(),
        email: "anders21@student.aau.dk".to_string(),
        username: "anders".to_string(),
        password: "".to_string(),
    };

    mock_services
        .user_context_mock
        .expect_create()
        .with(predicate::eq(user.clone()))
        .returning(move |_| Err(DbErr::RecordNotInserted)); //todo!("Needs to be a SqlError with UniqueConstraintViolation with 'username' in message)

    let api = get_mock_concrete_ecdar_api(mock_services);

    let create_user_request = Request::new(CreateUserRequest {
        email: "anders21@student.aau.dk".to_string(),
        username: "anders".to_string(),
        password: "".to_string(),
    });

    let res = api.create_user(create_user_request).await;
    assert_eq!(res.unwrap_err().code(), Code::Internal); //todo!("Needs to be code AlreadyExists when mocked Error is corrected)
}

#[tokio::test]
async fn create_user_invalid_username_returns_error() {
    let mock_services = get_mock_services();

    let api = get_mock_concrete_ecdar_api(mock_services);

    let create_user_request = Request::new(CreateUserRequest {
        email: "valid@email.com".to_string(),
        username: "ØØØØØ".to_string(),
        password: "123".to_string(),
    });

    let res = api.create_user(create_user_request).await;
    assert_eq!(res.unwrap_err().code(), Code::InvalidArgument);
}

#[tokio::test]
async fn test_create_user_valid_request_returns_ok() {
    let mut mock_services = get_mock_services();

    let user = user::Model {
        id: Default::default(),
        email: "newuser@example.com".to_string(),
        username: "newuser".to_string(),
        password: "StrongPassword123".to_string(),
    };

    mock_services
        .user_context_mock
        .expect_create()
        .with(predicate::eq(user.clone()))
        .returning(move |_| Ok(user.clone()));

    let api = get_mock_concrete_ecdar_api(mock_services);

    let create_user_request = Request::new(CreateUserRequest {
        email: "newuser@example.com".to_string(),
        username: "newuser".to_string(),
        password: "StrongPassword123".to_string(),
    });

    let create_user_response = api.create_user(create_user_request).await;
    assert!(create_user_response.is_ok());
}

#[tokio::test]
async fn update_user_returns_ok() {
    let mut mock_services = get_mock_services();

    let old_user = user::Model {
        id: 1,
        email: "olduser@example.com".to_string(),
        username: "old_username".to_string(),
        password: "StrongPassword123".to_string(),
    };

    let new_user = user::Model {
        id: 1,
        email: "newuser@example.com".to_string(),
        username: "new_username".to_string(),
        password: "g76df2gd7hd837g8hjd8723hd8gd823d82d3".to_string(),
    };

    mock_services
        .user_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| Ok(Some(old_user.clone())));

    mock_services
        .hashing_context_mock
        .expect_hash_password()
        .with(predicate::eq("StrongPassword123".to_string()))
        .returning(move |_| "g76df2gd7hd837g8hjd8723hd8gd823d82d3".to_string());

    mock_services
        .user_context_mock
        .expect_update()
        .with(predicate::eq(new_user.clone()))
        .returning(move |_| Ok(new_user.clone()));

    let api = get_mock_concrete_ecdar_api(mock_services);

    let mut update_user_request = Request::new(UpdateUserRequest {
        email: Some("newuser@example.com".to_string()),
        username: Some("new_username".to_string()),
        password: Some("StrongPassword123".to_string()),
    });

    update_user_request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let update_user_response = api.update_user(update_user_request).await;

    assert!(update_user_response.is_ok())
}

#[tokio::test]
async fn update_user_non_existant_user_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .user_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| Err(DbErr::RecordNotFound("".to_string())));

    let api = get_mock_concrete_ecdar_api(mock_services);

    let mut update_user_request = Request::new(UpdateUserRequest {
        email: Some("new_test@test".to_string()),
        username: Some("new_test_user".to_string()),
        password: Some("new_test_pass".to_string()),
    });

    update_user_request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let res = api.update_user(update_user_request).await;

    assert_eq!(res.unwrap_err().code(), Code::Internal);
}
