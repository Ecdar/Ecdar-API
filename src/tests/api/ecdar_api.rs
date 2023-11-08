#[cfg(test)]
mod ecdar_api {
    use crate::api::ecdar_api::helpers::helpers::AnyEntity;
    use crate::api::ecdar_api::ConcreteEcdarApi;
    use crate::api::server::server::ecdar_api_auth_server::EcdarApiAuth;
    use crate::api::server::server::{CreateUserRequest, DeleteUserRequest};
    use crate::database::entity_context::EntityContextTrait;
    use crate::{
        api::server::server::ecdar_api_server::EcdarApi,
        entities::user::Model as User,
    };
    use std::str::FromStr;
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
    async fn create_user_returns_token() -> () {
        let api = ConcreteEcdarApi::setup_in_memory_db(vec![AnyEntity::User]).await;

        let create_user_request = Request::new(CreateUserRequest {
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: "123".to_string(),
        });

        let create_user_response = api.create_user(create_user_request).await;
        assert!(create_user_response.is_ok())
    }

    #[tokio::test]
    async fn create_user_existing_user_returns_err() -> () {
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

        let create_user_request = Request::new(CreateUserRequest {
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: "123".to_string(),
        });

        let create_user_response = api.create_user(create_user_request).await;
        assert!(create_user_response.is_err())
    }
}
