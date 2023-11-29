use std::str::FromStr;

use mockall::predicate;
use tonic::{metadata, Code, Request};

use crate::{
    api::server::server::{ecdar_api_server::EcdarApi, DeleteModelRequest, ModelInfo},
    entities::model,
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
async fn list_model_info_returns_ok() {
    let mut mock_services = get_mock_services();

    let model_info = ModelInfo {
        model_id: 1,
        model_name: "model::Model name".to_owned(),
        model_owner_id: 1,
        user_role_on_model: "Editor".to_owned(),
    };

    mock_services
        .model_context_mock
        .expect_get_model_info_by_uid()
        .with(predicate::eq(1))
        .returning(move |_| Ok(vec![model_info.clone()]));

    let mut list_model_info_request = Request::new(());

    list_model_info_request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.list_model_info(list_model_info_request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn list_model_info_returns_err() {
    let mut mock_services = get_mock_services();

    mock_services
        .model_context_mock
        .expect_get_model_info_by_uid()
        .with(predicate::eq(1))
        .returning(move |_| Ok(vec![]));

    let mut list_model_info_request = Request::new(());

    list_model_info_request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.list_model_info(list_model_info_request).await;

    assert!(res.is_err());
}
