use std::str::FromStr;

use mockall::predicate;
use sea_orm::DbErr;
use tonic::{metadata, Code, Request};

use crate::api::auth::TokenType;
use crate::entities::{access, in_use, session};
use crate::{
    api::server::server::{
        ecdar_api_server::EcdarApi, ComponentsInfo, CreateModelRequest, DeleteModelRequest,
    },
    entities::model,
    tests::api::helpers::{get_mock_concrete_ecdar_api, get_mock_services},
};

#[tokio::test]
async fn create_model_returns_ok() {
    let mut mock_services = get_mock_services();

    let uid = 0;

    let components_info = ComponentsInfo {
        components: vec![],
        components_hash: 0,
    };

    let model = model::Model {
        id: Default::default(),
        name: Default::default(),
        components_info: serde_json::to_value(components_info.clone()).unwrap(),
        owner_id: uid,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        user_id: uid,
        model_id: model.id,
    };

    let session = session::Model {
        id: Default::default(),
        refresh_token: "refresh_token".to_string(),
        access_token: "access_token".to_string(),
        updated_at: Default::default(),
        user_id: uid,
    };

    let in_use = in_use::Model {
        model_id: model.id,
        session_id: session.id,
        latest_activity: Default::default(),
    };

    mock_services
        .model_context_mock
        .expect_create()
        .with(predicate::eq(model.clone()))
        .returning(move |_| Ok(model.clone()));

    mock_services
        .access_context_mock
        .expect_create()
        .with(predicate::eq(access.clone()))
        .returning(move |_| Ok(access.clone()));

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| Ok(Some(session.clone())));

    mock_services
        .in_use_context_mock
        .expect_create()
        .with(predicate::eq(in_use.clone()))
        .returning(move |_| Ok(in_use.clone()));

    let mut request = Request::new(CreateModelRequest {
        name: Default::default(),
        components_info: Option::from(components_info),
    });

    request
        .metadata_mut()
        .insert("uid", uid.to_string().parse().unwrap());

    request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
    );

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.create_model(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn create_model_existing_name_returns_err() {
    let mut mock_services = get_mock_services();

    let uid = 0;

    let model = model::Model {
        id: Default::default(),
        name: "model".to_string(),
        components_info: Default::default(),
        owner_id: uid,
    };

    mock_services
        .model_context_mock
        .expect_create()
        .with(predicate::eq(model.clone()))
        .returning(move |_| Err(DbErr::RecordNotInserted)); //todo!("Needs to be a SqlError with UniqueConstraintViolation with 'name' in message)

    let mut request = Request::new(CreateModelRequest {
        name: "model".to_string(),
        components_info: Default::default(),
    });

    request
        .metadata_mut()
        .insert("uid", uid.to_string().parse().unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.create_model(request).await;

    assert_eq!(res.unwrap_err().code(), Code::InvalidArgument); //todo!("Needs to be code AlreadyExists when mocked Error is corrected)
}

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
