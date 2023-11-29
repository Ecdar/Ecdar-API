use chrono::Utc;
use mockall::predicate;
use tonic::{Code, Request};

use crate::{
    api::{
        auth::TokenType,
        server::server::{ecdar_api_server::EcdarApi, GetModelRequest},
    },
    entities::{access, in_use, model, query, session},
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
        latest_activity: Utc::now().naive_utc(),
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
        latest_activity: Utc::now().naive_utc(),
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

    let in_use = in_use::Model {
        model_id: 0,
        session_id: 0,
        latest_activity: Default::default(),
    };

    let updated_in_use = in_use::Model {
        model_id: 0,
        session_id: 1,
        latest_activity: Default::default(),
    };

    let session = session::Model {
        id: 0,
        refresh_token: "refresh_token".to_owned(),
        access_token: "access_token".to_owned(),
        updated_at: Default::default(),
        user_id: Default::default(),
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
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_owned()),
        )
        .returning(move |_, _| Ok(Some(session.clone())));

    mock_services
        .query_context_mock
        .expect_get_all_by_model_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    mock_services
        .in_use_context_mock
        .expect_update()
        .returning(move |_| Ok(updated_in_use.clone()));

    let mut request = Request::new(GetModelRequest { id: 0 });

    request
        .metadata_mut()
        .insert("authorization", "Bearer access_token".parse().unwrap());
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
        latest_activity: Utc::now().naive_utc(),
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
        latest_activity: Utc::now().naive_utc(),
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
