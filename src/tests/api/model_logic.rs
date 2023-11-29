use std::str::FromStr;

use chrono::{Duration, Utc};
use mockall::predicate;
use sea_orm::DbErr;
use tonic::{metadata, Code, Request};

use crate::{
    api::{server::server::{ecdar_api_server::EcdarApi, DeleteModelRequest, ModelInfo, UpdateModelRequest, ComponentsInfo, Component, component::Rep, CreateModelRequest}, auth::TokenType},
    entities::{model, access, session, in_use},
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
async fn update_model_name_returns_ok() {
    let mut mock_services = get_mock_services();

    let user_id = 1;
    let model_id = 1;
    let new_model_name = "new_name".to_string();

    let mut update_model_request = Request::new(UpdateModelRequest {
        id: model_id.clone(),
        name: Some(new_model_name.clone()),
        components_info: None,
        owner_id: None,
    });

    update_model_request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
    );

    update_model_request.metadata_mut().insert(
        "uid", metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
    );

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(model_id))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: model_id,
                name: "old_name".to_owned(),
                components_info: Default::default(),
                owner_id: user_id,
            }))
        });

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(1), predicate::eq(model_id))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: user_id,
                model_id: model_id,
                role: "Editor".to_string(),
            }))
    });

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| Ok(Some(session::Model {
            id: 1,
            refresh_token: "refresh_token".to_string(),
            access_token: "access_token".to_string(),
            updated_at: Utc::now().naive_utc()-Duration::seconds(100),
            user_id: user_id,
        })));

    mock_services
        .model_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(model::Model {
                id: model_id,
                name: new_model_name.clone(),
                components_info: Default::default(),
                owner_id: user_id,
            })
        });

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .returning(move |_| {
            Ok(Some(in_use::Model {
                model_id: model_id,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            }))
        });

    mock_services
        .in_use_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(in_use::Model {
                model_id: 1,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            })
        });

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(update_model_request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn update_model_components_info_returns_ok() {
    let mut mock_services = get_mock_services();

    let user_id = 1;
    let model_id = 1;
    let components_info_non_json = ComponentsInfo {
        components: vec![
            Component {
               rep: Some(Rep::Json("a".to_owned())),     
            }
        ],
        components_hash: 1234456,
    };
    let components_info = serde_json::to_value(components_info_non_json.clone()).unwrap();

    let mut update_model_request = Request::new(UpdateModelRequest {
        id: model_id.clone(),
        name: None,
        components_info: Some(components_info_non_json.clone()),
        owner_id: None,
    });

    update_model_request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
    );

    update_model_request.metadata_mut().insert(
        "uid", metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
    );

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(model_id))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: model_id,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: user_id,
            }))
        });

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(1), predicate::eq(model_id))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: user_id,
                model_id: model_id,
                role: "Editor".to_string(),
            }))
    });

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| Ok(Some(session::Model {
            id: 1,
            refresh_token: "refresh_token".to_string(),
            access_token: "access_token".to_string(),
            updated_at: Utc::now().naive_utc()-Duration::seconds(100),
            user_id: user_id,
        })));

    mock_services
        .model_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(model::Model {
                id: model_id,
                name: Default::default(),
                components_info: components_info.clone(),
                owner_id: user_id,
            })
        });

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .returning(move |_| {
            Ok(Some(in_use::Model {
                model_id: model_id,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            }))
        });

    mock_services
        .in_use_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(in_use::Model {
                model_id: 1,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            })
        });

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(update_model_request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn update_model_owner_id_returns_ok() {
    let mut mock_services = get_mock_services();

    let user_id = 1;
    let model_id = 1;
    let new_owner_id = 2;

    let mut update_model_request = Request::new(UpdateModelRequest {
        id: model_id.clone(),
        name: None,
        components_info: None,
        owner_id: Some(new_owner_id.clone()),
    });

    update_model_request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
    );

    update_model_request.metadata_mut().insert(
        "uid", metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
    );

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(model_id))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: model_id,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: user_id,
            }))
        });

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(1), predicate::eq(model_id))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: user_id,
                model_id: model_id,
                role: "Editor".to_string(),
            }))
    });

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| Ok(Some(session::Model {
            id: 1,
            refresh_token: "refresh_token".to_string(),
            access_token: "access_token".to_string(),
            updated_at: Utc::now().naive_utc()-Duration::seconds(100),
            user_id: user_id,
        })));

    mock_services
        .model_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(model::Model {
                id: model_id,
                name: Default::default(),
                components_info: Default::default(),
                owner_id: new_owner_id.clone(),
            })
        });

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .returning(move |_| {
            Ok(Some(in_use::Model {
                model_id: model_id,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            }))
        });

    mock_services
        .in_use_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(in_use::Model {
                model_id: 1,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            })
        });

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(update_model_request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn update_model_returns_ok() {
    let mut mock_services = get_mock_services();

    let user_id = 1;
    let model_id = 1;
    let new_model_name = "new_name".to_string();
    let new_components_info_non_json = ComponentsInfo {
        components: vec![
            Component {
               rep: Some(Rep::Json("a".to_owned())),     
            }
        ],
        components_hash: 1234456,
    };
    let new_components_info = serde_json::to_value(new_components_info_non_json.clone()).unwrap();
    let new_owner_id = 2;

    let mut update_model_request = Request::new(UpdateModelRequest {
        id: model_id.clone(),
        name: Some(new_model_name.clone()),
        components_info: Some(new_components_info_non_json.clone()),
        owner_id: Some(new_owner_id.clone()),
    });

    update_model_request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
    );

    update_model_request.metadata_mut().insert(
        "uid", metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
    );

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(model_id))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: model_id,
                name: "old_name".to_owned(),
                components_info: serde_json::to_value("{\"old_components\":1}".clone()).unwrap(),
                owner_id: user_id,
            }))
        });

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(1), predicate::eq(model_id))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: user_id,
                model_id: model_id,
                role: "Editor".to_string(),
            }))
    });

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| Ok(Some(session::Model {
            id: 1,
            refresh_token: "refresh_token".to_string(),
            access_token: "access_token".to_string(),
            updated_at: Utc::now().naive_utc()-Duration::seconds(100),
            user_id: user_id,
        })));

    mock_services
        .model_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(model::Model {
                id: model_id,
                name: new_model_name.clone(),
                components_info: new_components_info.clone(),
                owner_id: new_owner_id.clone(),
            })
        });

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .returning(move |_| {
            Ok(Some(in_use::Model {
                model_id: model_id,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            }))
        });

    mock_services
        .in_use_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(in_use::Model {
                model_id: 1,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            })
        });

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(update_model_request).await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn update_model_owner_id_not_owner_returns_err() {
    let mut mock_services = get_mock_services();

    let user_id = 1;
    let model_id = 1;
    let new_model_name = "new_name".to_string();
    let new_components_info_non_json = ComponentsInfo {
        components: vec![
            Component {
               rep: Some(Rep::Json("a".to_owned())),     
            }
        ],
        components_hash: 1234456,
    };
    let new_components_info = serde_json::to_value(new_components_info_non_json.clone()).unwrap();
    let new_owner_id = 2;

    let mut update_model_request = Request::new(UpdateModelRequest {
        id: model_id.clone(),
        name: Some(new_model_name.clone()),
        components_info: Some(new_components_info_non_json.clone()),
        owner_id: Some(new_owner_id.clone()),
    });

    update_model_request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
    );

    update_model_request.metadata_mut().insert(
        "uid", metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
    );

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(model_id))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: model_id,
                name: "old_name".to_owned(),
                components_info: serde_json::to_value("{\"old_components\":1}".clone()).unwrap(),
                owner_id: 100,
            }))
        });

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(1), predicate::eq(model_id))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: user_id,
                model_id: model_id,
                role: "Editor".to_string(),
            }))
    });

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| Ok(Some(session::Model {
            id: 1,
            refresh_token: "refresh_token".to_string(),
            access_token: "access_token".to_string(),
            updated_at: Utc::now().naive_utc()-Duration::seconds(100),
            user_id: user_id,
        })));

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .returning(move |_| {
            Ok(Some(in_use::Model {
                model_id: model_id,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            }))
        });

    mock_services
        .in_use_context_mock
        .expect_update()
        .returning(move |_| {
            Ok(in_use::Model {
                model_id: 1,
                session_id: 1,
                latest_activity: Utc::now().naive_utc(),
            })
        });

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(update_model_request).await.unwrap_err();

    assert_eq!(res.code(), Code::PermissionDenied);

}

#[tokio::test]
async fn update_model_no_in_use_returns_err() {
    let mut mock_services = get_mock_services();

    let user_id = 1;
    let model_id = 1;
    let new_model_name = "new_name".to_string();
    let new_components_info_non_json = ComponentsInfo {
        components: vec![
            Component {
               rep: Some(Rep::Json("a".to_owned())),     
            }
        ],
        components_hash: 1234456,
    };
    let new_components_info = serde_json::to_value(new_components_info_non_json.clone()).unwrap();
    let new_owner_id = 2;

    let mut update_model_request = Request::new(UpdateModelRequest {
        id: model_id.clone(),
        name: Some(new_model_name.clone()),
        components_info: Some(new_components_info_non_json.clone()),
        owner_id: Some(new_owner_id.clone()),
    });

    update_model_request.metadata_mut().insert(
        "authorization",
        metadata::MetadataValue::from_str("Bearer access_token").unwrap(),
    );

    update_model_request.metadata_mut().insert(
        "uid", metadata::MetadataValue::from_str(user_id.to_string().as_str()).unwrap(),
    );

    mock_services
        .model_context_mock
        .expect_get_by_id()
        .with(predicate::eq(model_id))
        .returning(move |_| {
            Ok(Some(model::Model {
                id: model_id,
                name: "old_name".to_owned(),
                components_info: serde_json::to_value("{\"old_components\":1}".clone()).unwrap(),
                owner_id: 100,
            }))
        });

    mock_services
        .access_context_mock
        .expect_get_access_by_uid_and_model_id()
        .with(predicate::eq(1), predicate::eq(model_id))
        .returning(move |_, _| {
            Ok(Some(access::Model {
                id: 1,
                user_id: user_id,
                model_id: model_id,
                role: "Editor".to_string(),
            }))
    });

    mock_services
        .session_context_mock
        .expect_get_by_token()
        .with(
            predicate::eq(TokenType::AccessToken),
            predicate::eq("access_token".to_string()),
        )
        .returning(move |_, _| Ok(Some(session::Model {
            id: 1,
            refresh_token: "refresh_token".to_string(),
            access_token: "access_token".to_string(),
            updated_at: Utc::now().naive_utc()-Duration::seconds(100),
            user_id: user_id,
        })));

    mock_services
        .in_use_context_mock
        .expect_get_by_id()
        .returning(move |_| {
            Ok(Some(in_use::Model {
                model_id: model_id,
                session_id: 10,
                latest_activity: Utc::now().naive_utc(),
            }))
        });

    let api = get_mock_concrete_ecdar_api(mock_services);

    let res = api.update_model(update_model_request).await.unwrap_err();

    assert_eq!(res.code(), Code::FailedPrecondition);
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
