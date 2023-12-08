use crate::api::server::server::create_access_request::User;
use crate::api::server::server::{
    AccessInfo, CreateAccessRequest, DeleteAccessRequest, ListAccessInfoRequest,
    UpdateAccessRequest,
};
use crate::controllers::controller_impls::AccessController;
use crate::controllers::controller_traits::AccessControllerTrait;
use crate::entities::{access, project, user};
use crate::tests::controllers::helpers::{disguise_context_mocks, get_mock_contexts};
use mockall::predicate;
use sea_orm::DbErr;
use std::str::FromStr;
use tonic::{metadata, Code, Request};

#[tokio::test]
async fn create_invalid_access_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: 1,
        user_id: 1,
    };

    mock_contexts
        .access_context_mock
        .expect_create()
        .with(predicate::eq(access.clone()))
        .returning(move |_| Err(DbErr::RecordNotInserted));

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: Default::default(),
                role: "Editor".to_owned(),
                user_id: 1,
                project_id: 1,
            }))
        });

    mock_contexts
        .user_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(user::Model {
                id: 1,
                email: Default::default(),
                username: "test".to_string(),
                password: "test".to_string(),
            }))
        });

    let mut request = Request::new(CreateAccessRequest {
        role: "Editor".to_string(),
        project_id: 1,
        user: Some(User::UserId(1)),
    });

    request.metadata_mut().insert(
        "uid",
        tonic::metadata::MetadataValue::from_str("1").unwrap(),
    );

    let contexts = disguise_context_mocks(mock_contexts);
    let access_logic = AccessController::new(contexts);

    let res = access_logic.create_access(request).await.unwrap_err();

    assert_eq!(res.code(), Code::Internal);
}

#[tokio::test]
async fn create_access_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

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
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: Default::default(),
                role: "Editor".to_string(),
                user_id: 1,
                project_id: 1,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_create()
        .with(predicate::eq(access.clone()))
        .returning(move |_| Ok(access.clone()));

    mock_contexts
        .user_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(user::Model {
                id: 1,
                email: Default::default(),
                username: "test".to_string(),
                password: "test".to_string(),
            }))
        });

    let mut request = Request::new(CreateAccessRequest {
        role: "Editor".to_string(),
        project_id: 1,
        user: Some(User::UserId(1)),
    });

    request.metadata_mut().insert(
        "uid",
        tonic::metadata::MetadataValue::from_str("1").unwrap(),
    );

    let contexts = disguise_context_mocks(mock_contexts);
    let access_logic = AccessController::new(contexts);

    let res = access_logic.create_access(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn update_invalid_access_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    let access = access::Model {
        id: 2,
        role: "Editor".to_string(),
        project_id: Default::default(),
        user_id: Default::default(),
    };

    mock_contexts
        .access_context_mock
        .expect_update()
        .with(predicate::eq(access.clone()))
        .returning(move |_| Err(DbErr::RecordNotUpdated));

    mock_contexts
        .access_context_mock
        .expect_get_by_id()
        .with(predicate::eq(2))
        .returning(move |_| {
            Ok(Some(access::Model {
                id: 1,
                role: "Editor".to_string(),
                project_id: 1,
                user_id: 2,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                role: "Editor".to_string(),
                project_id: 1,
                user_id: 1,
            }))
        });

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: 1,
                name: "test".to_string(),
                owner_id: 1,
                components_info: Default::default(),
            }))
        });

    let mut request = Request::new(UpdateAccessRequest {
        id: 2,
        role: "Editor".to_string(),
    });

    request.metadata_mut().insert(
        "uid",
        tonic::metadata::MetadataValue::from_str("1").unwrap(),
    );

    let contexts = disguise_context_mocks(mock_contexts);
    let access_logic = AccessController::new(contexts);

    let res = access_logic.update_access(request).await.unwrap_err();

    assert_eq!(res.code(), Code::Internal);
}

#[tokio::test]
async fn update_access_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

    let access = access::Model {
        id: 2,
        role: "Editor".to_string(),
        project_id: Default::default(),
        user_id: Default::default(),
    };

    mock_contexts
        .access_context_mock
        .expect_update()
        .with(predicate::eq(access.clone()))
        .returning(move |_| Ok(access.clone()));

    mock_contexts
        .access_context_mock
        .expect_get_by_id()
        .with(predicate::eq(2))
        .returning(move |_| {
            Ok(Some(access::Model {
                id: 1,
                role: "Editor".to_string(),
                project_id: 1,
                user_id: 2,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                role: "Editor".to_string(),
                project_id: 1,
                user_id: 1,
            }))
        });

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: 1,
                name: "test".to_string(),
                owner_id: 1,
                components_info: Default::default(),
            }))
        });

    let mut request = Request::new(UpdateAccessRequest {
        id: 2,
        role: "Editor".to_string(),
    });

    request.metadata_mut().insert(
        "uid",
        tonic::metadata::MetadataValue::from_str("1").unwrap(),
    );

    let contexts = disguise_context_mocks(mock_contexts);
    let access_logic = AccessController::new(contexts);

    let res = access_logic.update_access(request).await;

    print!("{:?}", res);

    assert!(res.is_ok());
}

#[tokio::test]
async fn delete_invalid_access_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .access_context_mock
        .expect_delete()
        .with(predicate::eq(2))
        .returning(move |_| Err(DbErr::RecordNotFound("".to_string())));

    mock_contexts
        .access_context_mock
        .expect_get_by_id()
        .with(predicate::eq(2))
        .returning(move |_| {
            Ok(Some(access::Model {
                id: 1,
                role: "Editor".to_string(),
                project_id: 1,
                user_id: 2,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                role: "Editor".to_string(),
                project_id: 1,
                user_id: 1,
            }))
        });

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: 1,
                name: "test".to_string(),
                owner_id: 1,
                components_info: Default::default(),
            }))
        });

    let mut request = Request::new(DeleteAccessRequest { id: 2 });

    request.metadata_mut().insert(
        "uid",
        tonic::metadata::MetadataValue::from_str("1").unwrap(),
    );

    let contexts = disguise_context_mocks(mock_contexts);
    let access_logic = AccessController::new(contexts);

    let res = access_logic.delete_access(request).await.unwrap_err();

    assert_eq!(res.code(), Code::NotFound);
}

#[tokio::test]
async fn delete_access_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

    let access = access::Model {
        id: 2,
        role: "Editor".to_string(),
        project_id: Default::default(),
        user_id: Default::default(),
    };

    mock_contexts
        .access_context_mock
        .expect_delete()
        .with(predicate::eq(2))
        .returning(move |_| Ok(access.clone()));

    mock_contexts
        .access_context_mock
        .expect_get_by_id()
        .with(predicate::eq(2))
        .returning(move |_| {
            Ok(Some(access::Model {
                id: 1,
                role: "Editor".to_string(),
                project_id: 1,
                user_id: 2,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                role: "Editor".to_string(),
                project_id: 1,
                user_id: 1,
            }))
        });

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: 1,
                name: "test".to_string(),
                owner_id: 1,
                components_info: Default::default(),
            }))
        });

    let mut request = Request::new(DeleteAccessRequest { id: 2 });

    request.metadata_mut().insert(
        "uid",
        tonic::metadata::MetadataValue::from_str("1").unwrap(),
    );

    let contexts = disguise_context_mocks(mock_contexts);
    let access_logic = AccessController::new(contexts);

    let res = access_logic.delete_access(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn list_access_info_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

    let mut request: Request<ListAccessInfoRequest> =
        Request::new(ListAccessInfoRequest { project_id: 1 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let access = AccessInfo {
        id: 1,
        role: "Editor".to_string(),
        project_id: 1,
        user_id: 1,
    };

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                role: "Editor".to_string(),
                project_id: Default::default(),
                user_id: Default::default(),
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_project_id()
        .returning(move |_| Ok(vec![access.clone()]));

    let contexts = disguise_context_mocks(mock_contexts);
    let access_logic = AccessController::new(contexts);

    let res = access_logic.list_access_info(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn list_access_info_returns_not_found() {
    let mut mock_contexts = get_mock_contexts();

    let mut request = Request::new(ListAccessInfoRequest { project_id: 1 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let access = access::Model {
        id: 1,
        role: "Editor".to_string(),
        project_id: 1,
        user_id: 1,
    };

    mock_contexts
        .access_context_mock
        .expect_get_access_by_project_id()
        .returning(move |_| Ok(vec![]));

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .returning(move |_, _| Ok(Some(access.clone())));

    let contexts = disguise_context_mocks(mock_contexts);
    let access_logic = AccessController::new(contexts);

    let res = access_logic.list_access_info(request).await.unwrap_err();

    assert_eq!(res.code(), Code::NotFound);
}

#[tokio::test]
async fn list_access_info_returns_no_permission() {
    let mut request = Request::new(ListAccessInfoRequest { project_id: 1 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .returning(move |_, _| Ok(None));

    let contexts = disguise_context_mocks(mock_contexts);
    let access_logic = AccessController::new(contexts);

    let res = access_logic.list_access_info(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}
