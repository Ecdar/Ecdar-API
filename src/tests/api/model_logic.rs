use mockall::predicate;
use sea_orm::DbErr;
use tonic::{Code, Request};

use crate::{
    api::server::server::{ecdar_api_server::EcdarApi, ComponentsInfo, GetModelRequest},
    entities::{access, in_use, model, query},
    tests::api::helpers::{get_mock_concrete_ecdar_api, get_mock_services},
};

#[tokio::test]
async fn get_model_user_has_access_returns_ok() {
    let mut mock_services = get_mock_services();

    let model = model::Model {
        id: Default::default(),
        name: "model".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        model_id: 1,
        user_id: 1,
    };

    let in_use = in_use::Model {
        model_id: Default::default(),
        session_id: 0,
        latest_activity: Default::default(),
    };

    let queries: Vec<query::Model> = vec![];

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(model.clone())));

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(in_use.clone())));

    mock_services
        .query_context_mock
        .expect_get_all_by_model_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    let mut request = Request::new(GetModelRequest { id: 0 });

    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.get_model(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn get_model_user_has_no_access_returns_err() {
    let mut mock_services = get_mock_services();

    let model = model::Model {
        id: Default::default(),
        name: "model".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let in_use = in_use::Model {
        model_id: Default::default(),
        session_id: 0,
        latest_activity: Default::default(),
    };

    let queries: Vec<query::Model> = vec![];

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(None));

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(model.clone())));

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(in_use.clone())));

    mock_services
        .query_context_mock
        .expect_get_all_by_model_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    let mut request = Request::new(GetModelRequest { id: 0 });

    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.get_model(request).await.unwrap_err();

    assert!(dbg!(res.code()) == Code::PermissionDenied);
}

#[tokio::test]
async fn get_model_is_in_use_is_true() {
    let mut mock_services = get_mock_services();

    let model = model::Model {
        id: Default::default(),
        name: "model".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        model_id: 1,
        user_id: 1,
    };

    let in_use = in_use::Model {
        model_id: Default::default(),
        session_id: 0,
        latest_activity: Default::default(),
    };

    let queries: Vec<query::Model> = vec![];

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(model.clone())));

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(in_use.clone())));

    mock_services
        .query_context_mock
        .expect_get_all_by_model_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    let mut request = Request::new(GetModelRequest { id: 0 });

    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.get_model(request).await;

    assert!(res.unwrap().get_ref().in_use);
}

#[tokio::test]
async fn get_model_is_in_use_is_false() {
    let mut mock_services = get_mock_services();

    let model = model::Model {
        id: Default::default(),
        name: "model".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        model_id: 1,
        user_id: 1,
    };

    let queries: Vec<query::Model> = vec![];

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(model.clone())));

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(None));

    mock_services
        .query_context_mock
        .expect_get_all_by_model_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    let mut request = Request::new(GetModelRequest { id: 0 });

    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.get_model(request).await;

    assert!(!res.unwrap().get_ref().in_use);
}

#[tokio::test]
async fn get_model_model_has_no_queries_queries_are_empty() {
    let mut mock_services = get_mock_services();

    let model = model::Model {
        id: Default::default(),
        name: "model".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        model_id: 1,
        user_id: 1,
    };

    let in_use = in_use::Model {
        model_id: Default::default(),
        session_id: 0,
        latest_activity: Default::default(),
    };

    let query1 = query::Model {
        id: 0,
        model_id: 1,
        string: "query".to_owned(),
        result: None,
        outdated: false,
    };

    let query2 = query::Model {
        id: 1,
        model_id: 1,
        string: "query".to_owned(),
        result: Some(serde_json::to_value("result").unwrap()),
        outdated: false,
    };

    let queries: Vec<query::Model> = vec![];

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(model.clone())));

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(in_use.clone())));

    mock_services
        .query_context_mock
        .expect_get_all_by_model_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    let mut request = Request::new(GetModelRequest { id: 0 });

    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.get_model(request).await;

    assert!(res.unwrap().get_ref().queries.is_empty());
}

#[tokio::test]
async fn get_model_query_has_no_result_query_is_empty() {
    let mut mock_services = get_mock_services();

    let model = model::Model {
        id: Default::default(),
        name: "model".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        model_id: 1,
        user_id: 1,
    };

    let in_use = in_use::Model {
        model_id: Default::default(),
        session_id: 0,
        latest_activity: Default::default(),
    };

    let query = query::Model {
        id: 0,
        model_id: 1,
        string: "query".to_owned(),
        result: None,
        outdated: false,
    };

    let queries: Vec<query::Model> = vec![query];

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(model.clone())));

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(in_use.clone())));

    mock_services
        .query_context_mock
        .expect_get_all_by_model_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    let mut request = Request::new(GetModelRequest { id: 0 });

    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.get_model(request).await;

    assert!(res.unwrap().get_ref().queries[0].result.is_empty());
}
