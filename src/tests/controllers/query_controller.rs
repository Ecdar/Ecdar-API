use crate::api::server::server::query_response::{self, Result};
use crate::api::server::server::{
    CreateQueryRequest, DeleteQueryRequest, QueryResponse, SendQueryRequest, UpdateQueryRequest,
};
use crate::controllers::controller_impls::QueryController;
use crate::controllers::controller_traits::QueryControllerTrait;
use crate::entities::{access, project, query};
use crate::tests::controllers::helpers::{
    disguise_context_mocks, disguise_service_mocks, get_mock_contexts, get_mock_services,
};
use mockall::predicate;
use sea_orm::DbErr;
use std::str::FromStr;
use tonic::{metadata, Code, Request, Response};

#[tokio::test]
async fn create_invalid_query_returns_err() {
    let mut mock_contexts = get_mock_contexts();
    let mock_services = get_mock_services();

    let query = query::Model {
        id: Default::default(),
        string: "".to_string(),
        result: Default::default(),
        project_id: 1,
        outdated: Default::default(),
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: 1,
        user_id: 1,
    };

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .query_context_mock
        .expect_create()
        .with(predicate::eq(query.clone()))
        .returning(move |_| Err(DbErr::RecordNotInserted));

    let mut request = Request::new(CreateQueryRequest {
        string: "".to_string(),
        project_id: 1,
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let services = disguise_service_mocks(mock_services);
    let query_logic = QueryController::new(contexts, services);

    let res = query_logic.create_query(request).await.unwrap_err();

    assert_eq!(res.code(), Code::Internal);
}

#[tokio::test]
async fn create_query_returns_ok() {
    let mut mock_contexts = get_mock_contexts();
    let mock_services = get_mock_services();

    let query = query::Model {
        id: Default::default(),
        string: "".to_string(),
        result: Default::default(),
        project_id: 1,
        outdated: Default::default(),
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: 1,
        user_id: 1,
    };

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .query_context_mock
        .expect_create()
        .with(predicate::eq(query.clone()))
        .returning(move |_| Ok(query.clone()));

    let mut request = Request::new(CreateQueryRequest {
        string: "".to_string(),
        project_id: 1,
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let services = disguise_service_mocks(mock_services);
    let query_logic = QueryController::new(contexts, services);

    let res = query_logic.create_query(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn update_invalid_query_returns_err() {
    let mut mock_contexts = get_mock_contexts();
    let mock_services = get_mock_services();

    let old_query = query::Model {
        id: 1,
        string: "".to_string(),
        result: None,
        project_id: Default::default(),
        outdated: true,
    };

    let query = query::Model {
        string: "updated".to_string(),
        ..old_query.clone()
    };

    let access = access::Model {
        id: 1,
        role: "Editor".to_string(),
        project_id: Default::default(),
        user_id: 1,
    };

    mock_contexts
        .query_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| Ok(Some(old_query.clone())));

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .query_context_mock
        .expect_update()
        .with(predicate::eq(query.clone()))
        .returning(move |_| Err(DbErr::RecordNotUpdated));

    let mut request = Request::new(UpdateQueryRequest {
        id: 1,
        string: "updated".to_string(),
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let services = disguise_service_mocks(mock_services);
    let query_logic = QueryController::new(contexts, services);

    let res = query_logic.update_query(request).await.unwrap_err();

    assert_eq!(res.code(), Code::Internal);
}

#[tokio::test]
async fn update_query_returns_ok() {
    let mut mock_contexts = get_mock_contexts();
    let mock_services = get_mock_services();

    let old_query = query::Model {
        id: 1,
        string: "".to_string(),
        result: None,
        project_id: Default::default(),
        outdated: true,
    };

    let query = query::Model {
        string: "updated".to_string(),
        ..old_query.clone()
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: Default::default(),
        user_id: 1,
    };

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .query_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| Ok(Some(old_query.clone())));

    mock_contexts
        .query_context_mock
        .expect_update()
        .with(predicate::eq(query.clone()))
        .returning(move |_| Ok(query.clone()));

    let mut request = Request::new(UpdateQueryRequest {
        id: 1,
        string: "updated".to_string(),
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let services = disguise_service_mocks(mock_services);
    let query_logic = QueryController::new(contexts, services);

    let res = query_logic.update_query(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn delete_invalid_query_returns_err() {
    let mut mock_contexts = get_mock_contexts();
    let mock_services = get_mock_services();

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: Default::default(),
        user_id: 1,
    };

    let query = query::Model {
        id: 1,
        string: "".to_string(),
        result: Default::default(),
        project_id: Default::default(),
        outdated: Default::default(),
    };

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .query_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| Ok(Some(query.clone())));

    mock_contexts
        .query_context_mock
        .expect_delete()
        .with(predicate::eq(1))
        .returning(move |_| Err(DbErr::RecordNotFound("".to_string())));

    let mut request = Request::new(DeleteQueryRequest { id: 1 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let services = disguise_service_mocks(mock_services);
    let query_logic = QueryController::new(contexts, services);

    let res = query_logic.delete_query(request).await.unwrap_err();

    assert_eq!(res.code(), Code::NotFound);
}

#[tokio::test]
async fn delete_query_returns_ok() {
    let mut mock_contexts = get_mock_contexts();
    let mock_services = get_mock_services();

    let query = query::Model {
        id: 1,
        string: "".to_string(),
        result: Default::default(),
        project_id: Default::default(),
        outdated: Default::default(),
    };

    let query_clone = query.clone();

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: Default::default(),
        user_id: 1,
    };

    mock_contexts
        .query_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| Ok(Some(query.clone())));

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .query_context_mock
        .expect_delete()
        .with(predicate::eq(1))
        .returning(move |_| Ok(query_clone.clone()));

    let mut request = Request::new(DeleteQueryRequest { id: 1 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let services = disguise_service_mocks(mock_services);
    let query_logic = QueryController::new(contexts, services);

    let res = query_logic.delete_query(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn create_query_invalid_role_returns_err() {
    let mut mock_contexts = get_mock_contexts();
    let mock_services = get_mock_services();

    let query = query::Model {
        id: 1,
        string: "".to_string(),
        result: Default::default(),
        project_id: Default::default(),
        outdated: Default::default(),
    };

    let access = access::Model {
        id: Default::default(),
        role: "Viewer".to_string(),
        project_id: Default::default(),
        user_id: 1,
    };

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .query_context_mock
        .expect_create()
        .with(predicate::eq(query.clone()))
        .returning(move |_| Ok(query.clone()));

    let mut request = Request::new(CreateQueryRequest {
        string: "".to_string(),
        project_id: 1,
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let services = disguise_service_mocks(mock_services);
    let query_logic = QueryController::new(contexts, services);

    let res = query_logic.create_query(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn delete_query_invalid_role_returns_err() {
    let mut mock_contexts = get_mock_contexts();
    let mock_services = get_mock_services();

    let query = query::Model {
        id: 1,
        string: "".to_string(),
        result: Default::default(),
        project_id: Default::default(),
        outdated: Default::default(),
    };

    let query_clone = query.clone();

    let access = access::Model {
        id: Default::default(),
        role: "Viewer".to_string(),
        project_id: Default::default(),
        user_id: 1,
    };

    mock_contexts
        .query_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| Ok(Some(query.clone())));

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .query_context_mock
        .expect_delete()
        .with(predicate::eq(1))
        .returning(move |_| Ok(query_clone.clone()));

    let mut request = Request::new(DeleteQueryRequest { id: 1 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let services = disguise_service_mocks(mock_services);
    let query_logic = QueryController::new(contexts, services);

    let res = query_logic.delete_query(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn update_query_invalid_role_returns_err() {
    let mut mock_contexts = get_mock_contexts();
    let mock_services = get_mock_services();

    let old_query = query::Model {
        id: 1,
        string: "".to_string(),
        result: None,
        project_id: Default::default(),
        outdated: true,
    };

    let query = query::Model {
        string: "updated".to_string(),
        ..old_query.clone()
    };

    let access = access::Model {
        id: Default::default(),
        role: "Viewer".to_string(),
        project_id: Default::default(),
        user_id: 1,
    };

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .query_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| Ok(Some(old_query.clone())));

    mock_contexts
        .query_context_mock
        .expect_update()
        .with(predicate::eq(query.clone()))
        .returning(move |_| Ok(query.clone()));

    let mut request = Request::new(UpdateQueryRequest {
        id: 1,
        string: "updated".to_string(),
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let services = disguise_service_mocks(mock_services);
    let query_logic = QueryController::new(contexts, services);

    let res = query_logic.update_query(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn send_query_returns_ok() {
    let mut mock_contexts = get_mock_contexts();
    let mut mock_services = get_mock_services();

    let query = query::Model {
        id: Default::default(),
        string: "".to_string(),
        result: Default::default(),
        project_id: Default::default(),
        outdated: Default::default(),
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: Default::default(),
        user_id: 1,
    };

    let project = project::Model {
        id: Default::default(),
        name: "project".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let query_response = QueryResponse {
        query_id: Default::default(),
        info: Default::default(),
        result: Some(Result::Success(query_response::Success {})),
    };

    let updated_query = query::Model {
        result: Some(serde_json::to_value(query_response.clone().result).unwrap()),
        ..query.clone()
    };

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(project.clone())));

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .query_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(query.clone())));

    mock_services
        .reveaal_service_mock
        .expect_send_query()
        .returning(move |_| Ok(Response::new(query_response.clone())));

    mock_contexts
        .query_context_mock
        .expect_update()
        .with(predicate::eq(updated_query.clone()))
        .returning(move |_| Ok(updated_query.clone()));

    let mut request = Request::new(SendQueryRequest {
        id: Default::default(),
        project_id: Default::default(),
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let services = disguise_service_mocks(mock_services);
    let query_logic = QueryController::new(contexts, services);

    let res = query_logic.send_query(request).await;

    assert!(res.is_ok());
}
