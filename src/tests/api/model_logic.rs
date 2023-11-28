#[cfg(test)]
use crate::api::server::server::ecdar_api_server::EcdarApi;
use crate::api::server::server::{
    CreateAccessRequest, DeleteAccessRequest, UpdateAccessRequest,
};
use crate::entities::model;
use crate::tests::api::helpers::{get_mock_concrete_ecdar_api, get_mock_services};
use mockall::predicate;
use sea_orm::DbErr;
use std::str::FromStr;
use crate::api::server::server::ModelInfo;

use tonic::{metadata, Code, Request};

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