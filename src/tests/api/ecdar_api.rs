#[cfg(test)]
mod ecdar_api {
    use std::str::FromStr;

    use tonic::{metadata, Request};

    use crate::{api::server::server::ecdar_api_server::EcdarApi, entities::user::Model as User};
    use crate::api::ecdar_api::ConcreteEcdarApi;
    use crate::api::server::server::DeleteUserRequest;

    #[tokio::test]
    async fn delete_user_nonexisting_user_returns_error() -> () {
        let api = ConcreteEcdarApi::setup_in_memory_db().await;

        let mut delete_request = Request::new(DeleteUserRequest {
            token: "ben".to_owned(),
        });

        // Insert token into request metadata
        delete_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let delete_response = api.delete_user(delete_request).await;

        assert!(delete_response.is_err());
    }

    #[tokio::test]
    async fn delete_user_existing_user_returns_ok() -> () {
        let api = ConcreteEcdarApi::setup_in_memory_db().await;

        let user = User {
            id: 1,
            email: "anders21@student.aau.dk".to_string(),
            username: "anders".to_string(),
            password: "123".to_string(),
        };

        let mut delete_request = Request::new(DeleteUserRequest {
            token: "shut ur ass up".to_owned(),
        });

        // Insert token into request metadata
        delete_request
            .metadata_mut()
            .insert("uid", metadata::MetadataValue::from_str("1").unwrap());

        let delete_response = api.delete_user(delete_request).await;

        assert!(delete_response.is_err());
    }
}
