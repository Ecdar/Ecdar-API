use std::str::FromStr;

use chrono::Utc;
use mockall::predicate;
use tonic::{metadata, Code, Request};

use crate::{
    api::{
        auth::TokenType,
        server::server::{
            ecdar_api_server::EcdarApi, DeleteModelRequest, ModelInfo, UpdateModelRequest,
        },
    },
    entities::{access, in_use, model, session},
    tests::api::helpers::{get_mock_concrete_ecdar_api, get_mock_services},
};

#[tokio::test]
async fn delete_not_owner_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 2,
            }))
        });

    let mut request = Request::new(DeleteModelRequest { id: 1 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.delete_model(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn delete_invalid_model_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(2))
        .returning(move |_| Ok(None));

    let mut request = Request::new(DeleteModelRequest { id: 2 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.delete_model(request).await.unwrap_err();

    assert_eq!(res.code(), Code::NotFound);
}

#[tokio::test]
async fn delete_model_returns_ok() {
    let mut mock_services = get_mock_services();

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            }))
        });

    mock_services
        .model_context_mock
        .expect_delete()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(model::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            })
        });

    let mut request = Request::new(DeleteModelRequest { id: 1 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.delete_model(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn list_models_info_returns_ok() {
    let mut mock_services = get_mock_services();

    let model_info = ModelInfo {
        model_id: 1,
        model_name: "model::Model name".to_owned(),
        model_owner_id: 1,
        user_role_on_model: "Editor".to_owned(),
    };

    mock_services
        .model_context_mock
        .expect_get_models_info_by_uid()
        .with(predicate::eq(1))
        .returning(move |_| Ok(vec![model_info.clone()]));

    let mut list_models_info_request = Request::new(());

    list_models_info_request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.list_models_info(list_models_info_request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn list_models_info_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .model_context_mock
        .expect_get_models_info_by_uid()
        .with(predicate::eq(1))
        .returning(move |_| Ok(vec![]));

    let mut list_models_info_request = Request::new(());

    list_models_info_request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.list_models_info(list_models_info_request).await;

    assert!(res.is_err());
}

#[tokio::test]
async fn update_owner_not_owner_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 2,
            }))
        });

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: 1,
                model_id: 1,
                role: "Editor".to_owned(),
            }))
        });

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| {
            Ok(Some(session::Model {
                id: 1,
                refresh_token: "refresh_token".to_string(),
                access_token: "access_token".to_string(),
                updated_at: Default::default(),
                user_id: 1,
            }))
        });

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(in_use::Model {
                session_id: 1,
                latest_activity: Default::default(),
                model_id: 1,
            }))
        });

    mock_services
        .in_use_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(in_use::Model {
                session_id: 1,
                latest_activity: Default::default(),
                model_id: 1,
            })
        });

    let mut request = Request::new(UpdateModelRequest {
        id: 1,
        name: None,
        components_info: None,
        owner_id: Some(1),
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("access_token").unwrap(),
    );

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn update_model_in_use_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            }))
        });

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: 1,
                model_id: 1,
                role: "Editor".to_owned(),
            }))
        });

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| {
            Ok(Some(session::Model {
                id: 1,
                refresh_token: "refresh_token".to_string(),
                access_token: "access_token".to_string(),
                updated_at: Default::default(),
                user_id: 1,
            }))
        });

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(in_use::Model {
                session_id: 2,
                latest_activity: Utc::now().naive_utc(),
                model_id: 1,
            }))
        });

    let mut request = Request::new(UpdateModelRequest {
        id: 1,
        name: None,
        components_info: None,
        owner_id: None,
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("access_token").unwrap(),
    );

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(request).await.unwrap_err();

    assert_eq!(res.code(), Code::FailedPrecondition);
}

#[tokio::test]
async fn update_no_access_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            }))
        });

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| Ok(None));

    let mut request = Request::new(UpdateModelRequest {
        id: 1,
        name: None,
        components_info: None,
        owner_id: None,
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn update_incorrect_role_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            }))
        });

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: 1,
                model_id: 1,
                role: "Viewer".to_owned(),
            }))
        });

    let mut request = Request::new(UpdateModelRequest {
        id: 1,
        name: None,
        components_info: None,
        owner_id: None,
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn update_no_session_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            }))
        });

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: 1,
                model_id: 1,
                role: "Editor".to_owned(),
            }))
        });

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| Ok(None));

    let mut request = Request::new(UpdateModelRequest {
        id: 1,
        name: None,
        components_info: None,
        owner_id: None,
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("access_token").unwrap(),
    );

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(request).await.unwrap_err();

    assert_eq!(res.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn update_no_model_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(2))
        .returning(move |_| Ok(None));

    let mut request = Request::new(UpdateModelRequest {
        id: 2,
        name: None,
        components_info: None,
        owner_id: None,
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(request).await.unwrap_err();

    assert_eq!(res.code(), Code::NotFound);
}
