#[cfg(test)]
mod ecdar_api {
    use crate::api::ecdar_api::helpers::helpers::AnyEntity;
    use crate::api::ecdar_api::ConcreteEcdarApi;
    use crate::api::server::server::ecdar_api_auth_server::EcdarApiAuth;
    use crate::api::server::server::{CreateUserRequest, DeleteUserRequest, UpdateUserRequest};
    use crate::database::entity_context::EntityContextTrait;
    use crate::database::user_context::UserContextTrait;
    use crate::{api::server::server::ecdar_api_server::EcdarApi, entities::user::Model as User};
    use std::ops::Deref;
    use std::str::FromStr;
    use tonic::codegen::Body;
    use tonic::{metadata, Request};

    #[tokio::test]
    async fn delete_user_nonexistent_user_returns_err() -> () {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let mut delete_request = Request::new(DeleteUserRequest {
            token: "token".to_owned(),
        });

        // Insert uid into request metadata
        delete_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let delete_response = api.delete_user(delete_request).await;

        assert!(delete_response.is_err());
    }

    #[tokio::test]
    async fn delete_user_existing_user_returns_ok() -> () {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let _ = api
            .user_context
            .create(User {
                id: Default::default(),
                email: "anders21@student.aau.dk".to_string(),
                username: "anders".to_string(),
                password: "123".to_string(),
            })
            .await
            .unwrap();

        let mut delete_request = Request::new(DeleteUserRequest {
            token: "token".to_owned(),
        });

        // Insert uid into request metadata
        delete_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let delete_response = api.delete_user(delete_request).await;

        assert!(delete_response.is_ok());
    }

    #[tokio::test]
    async fn create_user_nonexistent_user_returns_ok() {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let create_user_request = Request::new(CreateUserRequest {
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: "123".to_string(),
        });

        let create_user_response = api.create_user(create_user_request).await;
        assert!(create_user_response.is_ok());
    }

    #[tokio::test]
    async fn create_user_nonexistent_user_inserts_user() {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let username = "newuser".to_string();

        let create_user_request = Request::new(CreateUserRequest {
            email: "anders21@student.aau.dk".to_string(),
            username: username.clone(),
            password: "123".to_string(),
        });

        let _ = api.create_user(create_user_request).await;

        assert!(api
            .user_context
            .get_by_username(username)
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn test_create_user_duplicate_email_returns_error() {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let _ = api
            .user_context
            .create(User {
                id: Default::default(),
                email: "existing@example.com".to_string(),
                username: "newuser1".to_string(),
                password: "123".to_string(),
            })
            .await;

        let create_user_request = Request::new(CreateUserRequest {
            email: "existing@example.com".to_string(),
            username: "newuser2".to_string(),
            password: "123".to_string(),
        });

        let create_user_response = api.create_user(create_user_request).await;
        assert!(create_user_response.is_err());
    }

    #[tokio::test]
    async fn test_create_user_invalid_email_returns_error() {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let create_user_request = Request::new(CreateUserRequest {
            email: "invalid-email".to_string(),
            username: "newuser".to_string(),
            password: "123".to_string(),
        });

        let create_user_response = api.create_user(create_user_request).await;
        assert!(create_user_response.is_err());
    }

    #[tokio::test]
    async fn test_create_user_duplicate_username_returns_error() {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let _ = api
            .user_context
            .create(User {
                id: Default::default(),
                email: "valid@email.com".to_string(),
                username: "existing".to_string(),
                password: "123".to_string(),
            })
            .await;

        let create_user_request = Request::new(CreateUserRequest {
            email: "valid@email2.com".to_string(),
            username: "existing".to_string(),
            password: "123".to_string(),
        });

        let create_user_response = api.create_user(create_user_request).await;
        assert!(create_user_response.is_err());
    }

    #[tokio::test]
    async fn test_create_user_invalid_username_returns_error() {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let create_user_request = Request::new(CreateUserRequest {
            email: "valid@email.com".to_string(),
            username: "invalid username".to_string(),
            password: "123".to_string(),
        });

        let create_user_response = api.create_user(create_user_request).await;
        assert!(create_user_response.is_err());
    }

    #[tokio::test]
    async fn test_create_user_valid_request_returns_ok() {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let create_user_request = Request::new(CreateUserRequest {
            email: "newuser@example.com".to_string(),
            username: "newuser".to_string(),
            password: "StrongPassword123".to_string(),
        });

        let create_user_response = api.create_user(create_user_request).await;
        assert!(create_user_response.is_ok());
    }

    #[tokio::test]
    async fn update_user_returns_ok() -> () {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let user = User {
            id: 1,
            email: "test@test".to_string(),
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
        };

        api.user_context.create(user.clone()).await.unwrap();

        let mut update_user_request = Request::new(UpdateUserRequest {
            token: "token".to_string(),
            email: Some("new_test@test".to_string()),
            username: Some("new_test_user".to_string()),
            password: Some("new_test_pass".to_string()),
        });

        update_user_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let update_user_response = api.update_user(update_user_request).await;

        assert!(update_user_response.is_ok())
    }

    #[tokio::test]
    async fn update_user_non_existant_user_returns_err() -> () {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let mut update_user_request = Request::new(UpdateUserRequest {
            token: "token".to_string(),
            email: Some("new_test@test".to_string()),
            username: Some("new_test_user".to_string()),
            password: Some("new_test_pass".to_string()),
        });

        update_user_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let update_user_response = api.update_user(update_user_request).await;

        assert!(update_user_response.is_err())
    }

    #[tokio::test]
    async fn update_user_single_field_returns_ok() -> () {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let user = User {
            id: 1,
            email: "test@test".to_string(),
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
        };

        api.user_context.create(user.clone()).await.unwrap();

        let mut update_user_request = Request::new(UpdateUserRequest {
            token: "token".to_string(),
            email: Some("new_test@test".to_string()),
            username: None,
            password: None,
        });

        update_user_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let update_user_response = api.update_user(update_user_request).await;

        assert!(update_user_response.is_ok())
    }
}
