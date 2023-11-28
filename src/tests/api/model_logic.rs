#[cfg(test)]
use crate::api::server::server::ecdar_api_server::EcdarApi;
use crate::api::server::server::{Component, ComponentsInfo, CreateModelRequest};
use crate::entities::{model, user};
use crate::tests::api::helpers::{get_mock_concrete_ecdar_api, get_mock_services};
use mockall::predicate;
use sea_orm::DbErr;
use tonic::{Code, Request};

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
        owner_id: uid.clone(),
    };

    mock_services
        .model_context_mock
        .expect_create()
        .with(predicate::eq(model.clone()))
        .returning(move |_| Ok(model.clone()));

    let mut request = Request::new(CreateModelRequest {
        name: Default::default(),
        components_info: Option::from(components_info),
        owner_id: uid.clone(),
    });

    request
        .metadata_mut()
        .insert("uid", uid.to_string().parse().unwrap());

    println!("{:?}", request);

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
        owner_id: uid.clone(),
    };

    mock_services
        .model_context_mock
        .expect_create()
        .with(predicate::eq(model.clone()))
        .returning(move |_| Err(DbErr::RecordNotInserted)); //todo!("Needs to be a SqlError with UniqueConstraintViolation with 'name' in message)

    let mut request = Request::new(CreateModelRequest {
        name: "model".to_string(),
        components_info: Default::default(),
        owner_id: uid.clone(),
    });

    request
        .metadata_mut()
        .insert("uid", uid.to_string().parse().unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.create_model(request).await;

    assert_eq!(res.unwrap_err().code(), Code::InvalidArgument); //todo!("Needs to be code AlreadyExists when mocked Error is corrected)
}
