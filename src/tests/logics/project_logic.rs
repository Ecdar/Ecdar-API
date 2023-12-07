use crate::logics::logic_impls::ProjectLogic;
use crate::logics::logic_traits::ProjectLogicTrait;
use crate::tests::logics::helpers::disguise_context_mocks;
use crate::{
    api::{
        auth::TokenType,
        server::server::{
            component::Rep, Component, ComponentsInfo, CreateProjectRequest, DeleteProjectRequest,
            GetProjectRequest, ProjectInfo, UpdateProjectRequest,
        },
    },
    entities::{access, in_use, project, query, session},
    tests::logics::helpers::get_mock_contexts,
};
use chrono::Utc;
use mockall::predicate;
use sea_orm::DbErr;
use std::str::FromStr;
use tonic::{metadata, Code, Request};

#[tokio::test]
async fn create_project_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

    let uid = 0;

    let components_info = ComponentsInfo {
        components: vec![],
        components_hash: 0,
    };

    let project = project::Model {
        id: Default::default(),
        name: Default::default(),
        components_info: serde_json::to_value(components_info.clone()).unwrap(),
        owner_id: uid,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        user_id: uid,
        project_id: project.id,
    };

    let session = session::Model {
        id: Default::default(),
        refresh_token: "refresh_token".to_string(),
        access_token: "access_token".to_string(),
        updated_at: Default::default(),
        user_id: uid,
    };

    let in_use = in_use::Model {
        project_id: project.id,
        session_id: session.id,
        latest_activity: Default::default(),
    };

    mock_contexts
        .project_context_mock
        .expect_create()
        .with(predicate::eq(project.clone()))
        .returning(move |_| Ok(project.clone()));

    mock_contexts
        .access_context_mock
        .expect_create()
        .with(predicate::eq(access.clone()))
        .returning(move |_| Ok(access.clone()));

    mock_contexts
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| Ok(Some(session.clone())));

    mock_contexts
        .in_use_context_mock
        .expect_create()
        .with(predicate::eq(in_use.clone()))
        .returning(move |_| Ok(in_use.clone()));

    let mut request = Request::new(CreateProjectRequest {
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

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.create_project(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn create_project_existing_name_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    let uid = 0;

    let project = project::Model {
        id: Default::default(),
        name: "project".to_string(),
        components_info: Default::default(),
        owner_id: uid,
    };

    mock_contexts
        .project_context_mock
        .expect_create()
        .with(predicate::eq(project.clone()))
        .returning(move |_| Err(DbErr::RecordNotInserted)); //todo!("Needs to be a SqlError with UniqueConstraintViolation with 'name' in message)

    let mut request = Request::new(CreateProjectRequest {
        name: "project".to_string(),
        components_info: Default::default(),
    });

    request
        .metadata_mut()
        .insert("uid", uid.to_string().parse().unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.create_project(request).await;

    assert_eq!(res.unwrap_err().code(), Code::InvalidArgument); //todo!("Needs to be code AlreadyExists when mocked Error is corrected)
}

#[tokio::test]
async fn get_project_user_has_access_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

    let project = project::Model {
        id: Default::default(),
        name: "project".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: 1,
        user_id: 1,
    };

    let in_use = in_use::Model {
        project_id: Default::default(),
        session_id: 0,
        latest_activity: Utc::now().naive_utc(),
    };

    let queries: Vec<query::Model> = vec![];

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(project.clone())));

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(in_use.clone())));

    mock_contexts
        .query_context_mock
        .expect_get_all_by_project_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    let mut request = Request::new(GetProjectRequest { id: 0 });

    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.get_project(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn delete_not_owner_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 2,
            }))
        });

    let mut request = Request::new(DeleteProjectRequest { id: 1 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.delete_project(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn delete_invalid_project_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(2))
        .returning(move |_| Ok(None));

    let mut request = Request::new(DeleteProjectRequest { id: 2 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.delete_project(request).await.unwrap_err();

    assert_eq!(res.code(), Code::NotFound);
}

#[tokio::test]
async fn delete_project_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            }))
        });

    mock_contexts
        .project_context_mock
        .expect_delete()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(project::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            })
        });

    let mut request = Request::new(DeleteProjectRequest { id: 1 });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.delete_project(request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn get_project_user_has_no_access_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    let project = project::Model {
        id: Default::default(),
        name: "project".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let in_use = in_use::Model {
        project_id: Default::default(),
        session_id: 0,
        latest_activity: Default::default(),
    };

    let queries: Vec<query::Model> = vec![];

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(None));

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(project.clone())));

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(in_use.clone())));

    mock_contexts
        .query_context_mock
        .expect_get_all_by_project_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    let mut request = Request::new(GetProjectRequest { id: 0 });

    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.get_project(request).await.unwrap_err();

    assert!(res.code() == Code::PermissionDenied);
}

#[tokio::test]
async fn get_project_is_in_use_is_true() {
    let mut mock_contexts = get_mock_contexts();

    let project = project::Model {
        id: Default::default(),
        name: "project".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: 1,
        user_id: 1,
    };

    let in_use = in_use::Model {
        project_id: Default::default(),
        session_id: 0,
        latest_activity: Utc::now().naive_utc(),
    };

    let queries: Vec<query::Model> = vec![];

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(project.clone())));

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(in_use.clone())));

    mock_contexts
        .query_context_mock
        .expect_get_all_by_project_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    let mut request = Request::new(GetProjectRequest { id: 0 });

    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.get_project(request).await;

    assert!(res.unwrap().get_ref().in_use);
}

#[tokio::test]
async fn get_project_is_in_use_is_false() {
    let mut mock_contexts = get_mock_contexts();

    let project = project::Model {
        id: Default::default(),
        name: "project".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: 1,
        user_id: 1,
    };

    let in_use = in_use::Model {
        project_id: 0,
        session_id: 0,
        latest_activity: Default::default(),
    };

    let updated_in_use = in_use::Model {
        project_id: 0,
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

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(project.clone())));

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(in_use.clone())));

    mock_contexts
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_owned()),
        )
        .returning(move |_, _| Ok(Some(session.clone())));

    mock_contexts
        .query_context_mock
        .expect_get_all_by_project_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    mock_contexts
        .in_use_context_mock
        .expect_update()
        .returning(move |_| Ok(updated_in_use.clone()));

    let mut request = Request::new(GetProjectRequest { id: 0 });

    request
        .metadata_mut()
        .insert("authorization", "Bearer access_token".parse().unwrap());
    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.get_project(request).await;

    assert!(!res.unwrap().get_ref().in_use);
}

#[tokio::test]
async fn get_project_project_has_no_queries_queries_are_empty() {
    let mut mock_contexts = get_mock_contexts();

    let project = project::Model {
        id: Default::default(),
        name: "project".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: 1,
        user_id: 1,
    };

    let in_use = in_use::Model {
        project_id: Default::default(),
        session_id: 0,
        latest_activity: Utc::now().naive_utc(),
    };

    let queries: Vec<query::Model> = vec![];

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(project.clone())));

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(in_use.clone())));

    mock_contexts
        .query_context_mock
        .expect_get_all_by_project_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    let mut request = Request::new(GetProjectRequest { id: 0 });

    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.get_project(request).await;

    assert!(res.unwrap().get_ref().queries.is_empty());
}

#[tokio::test]
async fn get_project_query_has_no_result_query_is_empty() {
    let mut mock_contexts = get_mock_contexts();

    let project = project::Model {
        id: Default::default(),
        name: "project".to_string(),
        components_info: Default::default(),
        owner_id: 0,
    };

    let access = access::Model {
        id: Default::default(),
        role: "Editor".to_string(),
        project_id: 1,
        user_id: 1,
    };

    let in_use = in_use::Model {
        project_id: Default::default(),
        session_id: 0,
        latest_activity: Utc::now().naive_utc(),
    };

    let query = query::Model {
        id: 0,
        project_id: 1,
        string: "query".to_owned(),
        result: None,
        outdated: false,
    };

    let queries: Vec<query::Model> = vec![query];

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(0), predicate::eq(0))
        .returning(move |_, _| Ok(Some(access.clone())));

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(project.clone())));

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(Some(in_use.clone())));

    mock_contexts
        .query_context_mock
        .expect_get_all_by_project_id()
        .with(predicate::eq(0))
        .returning(move |_| Ok(queries.clone()));

    let mut request = Request::new(GetProjectRequest { id: 0 });

    request.metadata_mut().insert("uid", "0".parse().unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.get_project(request).await;

    assert!(res.unwrap().get_ref().queries[0].result.is_empty());
}

#[tokio::test]
async fn list_projects_info_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

    let project_info = ProjectInfo {
        project_id: 1,
        project_name: "project::Model name".to_owned(),
        project_owner_id: 1,
        user_role_on_project: "Editor".to_owned(),
    };

    mock_contexts
        .project_context_mock
        .expect_get_project_info_by_uid()
        .with(predicate::eq(1))
        .returning(move |_| Ok(vec![project_info.clone()]));

    let mut list_projects_info_request = Request::new(());

    list_projects_info_request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic
        .list_projects_info(list_projects_info_request)
        .await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn list_projects_info_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .project_context_mock
        .expect_get_project_info_by_uid()
        .with(predicate::eq(1))
        .returning(move |_| Ok(vec![]));

    let mut list_projects_info_request = Request::new(());

    list_projects_info_request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic
        .list_projects_info(list_projects_info_request)
        .await;

    assert!(res.is_err());
}

#[tokio::test]
async fn update_name_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

    let user_id = 1;
    let project_id = 1;
    let new_project_name = "new_name".to_string();

    let mut update_project_request = Request::new(UpdateProjectRequest {
        id: project_id,
        name: Some(new_project_name.clone()),
        components_info: None,
        owner_id: None,
    });

    update_project_request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
    );

    update_project_request.metadata_mut().insert(
        "uid",
        metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
    );

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(project_id))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: project_id,
                name: "old_name".to_owned(),
                components_info: Default::default(),
                owner_id: user_id,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(project_id))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id,
                project_id,
                role: "Editor".to_string(),
            }))
        });

    mock_contexts
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
                user_id,
            }))
        });

    mock_contexts
        .project_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(project::Model {
                id: project_id,
                name: new_project_name.clone(),
                components_info: Default::default(),
                owner_id: user_id,
            })
        });

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .returning(move |_| {
            Ok(Some(in_use::Model {
                project_id,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            }))
        });

    mock_contexts
        .in_use_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(in_use::Model {
                project_id: 1,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            })
        });

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.update_project(update_project_request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn update_components_info_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

    let user_id = 1;
    let project_id = 1;
    let components_info_non_json = ComponentsInfo {
        components: vec![Component {
            rep: Some(Rep::Json("a".to_owned())),
        }],
        components_hash: 1234456,
    };
    let components_info = serde_json::to_value(components_info_non_json.clone()).unwrap();

    let mut update_project_request = Request::new(UpdateProjectRequest {
        id: project_id,
        name: None,
        components_info: Some(components_info_non_json.clone()),
        owner_id: None,
    });

    update_project_request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
    );

    update_project_request.metadata_mut().insert(
        "uid",
        metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
    );

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(project_id))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: project_id,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: user_id,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(project_id))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id,
                project_id,
                role: "Editor".to_string(),
            }))
        });

    mock_contexts
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
                user_id,
            }))
        });

    mock_contexts
        .project_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(project::Model {
                id: project_id,
                name: Default::default(),
                components_info: components_info.clone(),
                owner_id: user_id,
            })
        });

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .returning(move |_| {
            Ok(Some(in_use::Model {
                project_id,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            }))
        });

    mock_contexts
        .in_use_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(in_use::Model {
                project_id: 1,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            })
        });

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.update_project(update_project_request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn update_owner_id_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

    let user_id = 1;
    let project_id = 1;
    let new_owner_id = 2;

    let mut update_project_request = Request::new(UpdateProjectRequest {
        id: project_id,
        name: None,
        components_info: None,
        owner_id: Some(new_owner_id),
    });

    update_project_request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
    );

    update_project_request.metadata_mut().insert(
        "uid",
        metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
    );

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(project_id))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: project_id,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: user_id,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(project_id))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id,
                project_id,
                role: "Editor".to_string(),
            }))
        });

    mock_contexts
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
                user_id,
            }))
        });

    mock_contexts
        .project_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(project::Model {
                id: project_id,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: new_owner_id,
            })
        });

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .returning(move |_| {
            Ok(Some(in_use::Model {
                project_id,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            }))
        });

    mock_contexts
        .in_use_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(in_use::Model {
                project_id: 1,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            })
        });

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.update_project(update_project_request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn update_returns_ok() {
    let mut mock_contexts = get_mock_contexts();

    let user_id = 1;
    let project_id = 1;
    let new_project_name = "new_name".to_string();
    let new_components_info_non_json = ComponentsInfo {
        components: vec![Component {
            rep: Some(Rep::Json("a".to_owned())),
        }],
        components_hash: 1234456,
    };
    let new_components_info = serde_json::to_value(new_components_info_non_json.clone()).unwrap();
    let new_owner_id = 2;

    let mut update_project_request = Request::new(UpdateProjectRequest {
        id: project_id,
        name: Some(new_project_name.clone()),
        components_info: Some(new_components_info_non_json.clone()),
        owner_id: Some(new_owner_id),
    });

    update_project_request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
    );

    update_project_request.metadata_mut().insert(
        "uid",
        metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
    );

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(project_id))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: project_id,
                name: "old_name".to_owned(),
                components_info: serde_json::to_value("{\"old_components\":1}".clone()).unwrap(),
                owner_id: user_id,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(project_id))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id,
                project_id,
                role: "Editor".to_string(),
            }))
        });

    mock_contexts
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
                user_id,
            }))
        });

    mock_contexts
        .project_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(project::Model {
                id: project_id,
                name: new_project_name.clone(),
                components_info: new_components_info.clone(),
                owner_id: new_owner_id,
            })
        });

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .returning(move |_| {
            Ok(Some(in_use::Model {
                project_id,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            }))
        });

    mock_contexts
        .in_use_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(in_use::Model {
                project_id: 1,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            })
        });

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.update_project(update_project_request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn update_owner_not_owner_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 2,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: 1,
                project_id: 1,
                role: "Editor".to_owned(),
            }))
        });

    mock_contexts
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

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(in_use::Model {
                session_id: 1,
                latest_activity: Default::default(),
                project_id: 1,
            }))
        });

    mock_contexts
        .in_use_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(in_use::Model {
                session_id: 1,
                latest_activity: Default::default(),
                project_id: 1,
            })
        });

    let mut request = Request::new(UpdateProjectRequest {
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

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.update_project(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn update_no_in_use_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: 1,
                project_id: 1,
                role: "Editor".to_owned(),
            }))
        });

    mock_contexts
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

    mock_contexts
        .in_use_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(in_use::Model {
                session_id: 2,
                latest_activity: Utc::now().naive_utc(),
                project_id: 1,
            }))
        });

    let mut request = Request::new(UpdateProjectRequest {
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

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.update_project(request).await.unwrap_err();

    assert_eq!(res.code(), Code::FailedPrecondition);
}

#[tokio::test]
async fn update_no_access_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| Ok(None));

    let mut request = Request::new(UpdateProjectRequest {
        id: 1,
        name: None,
        components_info: None,
        owner_id: None,
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.update_project(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn update_incorrect_role_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: 1,
                project_id: 1,
                role: "Viewer".to_owned(),
            }))
        });

    let mut request = Request::new(UpdateProjectRequest {
        id: 1,
        name: None,
        components_info: None,
        owner_id: None,
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.update_project(request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn update_no_session_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(1))
        .returning(move |_| {
            Ok(Some(project::Model {
                id: 1,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: 1,
            }))
        });

    mock_contexts
        .access_context_mock
        .expect_get_access_by_uid_and_project_id()
        .with(predicate::eq(1), predicate::eq(1))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: 1,
                project_id: 1,
                role: "Editor".to_owned(),
            }))
        });

    mock_contexts
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| Ok(None));

    let mut request = Request::new(UpdateProjectRequest {
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

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.update_project(request).await.unwrap_err();

    assert_eq!(res.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn update_no_project_returns_err() {
    let mut mock_contexts = get_mock_contexts();

    mock_contexts
        .project_context_mock
        .expect_get_by_id()
        .with(predicate::eq(2))
        .returning(move |_| Ok(None));

    let mut request = Request::new(UpdateProjectRequest {
        id: 2,
        name: None,
        components_info: None,
        owner_id: None,
    });

    request
        .metadata_mut()
        .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

    let contexts = disguise_context_mocks(mock_contexts);
    let project_logic = ProjectLogic::new(contexts);

    let res = project_logic.update_project(request).await.unwrap_err();

    assert_eq!(res.code(), Code::NotFound);
}
