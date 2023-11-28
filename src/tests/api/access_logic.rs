#[cfg(test)]
mod access_logic {
    use crate::api::server::server::ecdar_api_server::EcdarApi;
    use crate::api::server::server::{
        CreateAccessRequest, DeleteAccessRequest, UpdateAccessRequest,
    };
    use crate::entities::access;
    use crate::tests::api::helpers::{get_mock_concrete_ecdar_api, get_mock_services};
    use mockall::predicate;
    use sea_orm::DbErr;
    use tonic::{Code, Request};

    #[tokio::test]
    async fn create_invalid_access_returns_err() {
        let mut mock_services = get_mock_services();

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            model_id: 1,
            user_id: 1,
        };

        mock_services
            .access_context_mock
            .expect_create()
            .with(predicate::eq(access.clone()))
            .returning(move |_| Err(DbErr::RecordNotInserted));

        let request = Request::new(CreateAccessRequest {
            role: "Editor".to_string(),
            model_id: 1,
            user_id: 1,
        });

        let api = get_mock_concrete_ecdar_api(mock_services);

        let res = api.create_access(request).await.unwrap_err();

        assert_eq!(res.code(), Code::Internal);
    }

    #[tokio::test]
    async fn create_access_returns_ok() {
        let mut mock_services = get_mock_services();

        let access = access::Model {
            id: Default::default(),
            role: "Editor".to_string(),
            model_id: 1,
            user_id: 1,
        };

        mock_services
            .access_context_mock
            .expect_create()
            .with(predicate::eq(access.clone()))
            .returning(move |_| Ok(access.clone()));

        let request = Request::new(CreateAccessRequest {
            role: "Editor".to_string(),
            model_id: 1,
            user_id: 1,
        });

        let api = get_mock_concrete_ecdar_api(mock_services);

        let res = api.create_access(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn update_invalid_access_returns_err() {
        let mut mock_services = get_mock_services();

        let access = access::Model {
            id: 1,
            role: "Editor".to_string(),
            model_id: Default::default(),
            user_id: Default::default(),
        };

        mock_services
            .access_context_mock
            .expect_update()
            .with(predicate::eq(access.clone()))
            .returning(move |_| Err(DbErr::RecordNotUpdated));

        let request = Request::new(UpdateAccessRequest {
            id: 1,
            role: "Editor".to_string(),
        });

        let api = get_mock_concrete_ecdar_api(mock_services);

        let res = api.update_access(request).await.unwrap_err();

        assert_eq!(res.code(), Code::Internal);
    }

    #[tokio::test]
    async fn update_access_returns_ok() {
        let mut mock_services = get_mock_services();

        let access = access::Model {
            id: 1,
            role: "Editor".to_string(),
            model_id: Default::default(),
            user_id: Default::default(),
        };

        mock_services
            .access_context_mock
            .expect_update()
            .with(predicate::eq(access.clone()))
            .returning(move |_| Ok(access.clone()));

        let request = Request::new(UpdateAccessRequest {
            id: 1,
            role: "Editor".to_string(),
        });

        let api = get_mock_concrete_ecdar_api(mock_services);

        let res = api.update_access(request).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn delete_invalid_access_returns_err() {
        let mut mock_services = get_mock_services();

        mock_services
            .access_context_mock
            .expect_delete()
            .with(predicate::eq(1))
            .returning(move |_| Err(DbErr::RecordNotFound("".to_string())));

        let request = Request::new(DeleteAccessRequest { id: 1 });

        let api = get_mock_concrete_ecdar_api(mock_services);

        let res = api.delete_access(request).await.unwrap_err();

        assert_eq!(res.code(), Code::NotFound);
    }

    #[tokio::test]
    async fn delete_access_returns_ok() {
        let mut mock_services = get_mock_services();

        let access = access::Model {
            id: 1,
            role: "Editor".to_string(),
            model_id: Default::default(),
            user_id: Default::default(),
        };

        mock_services
            .access_context_mock
            .expect_delete()
            .with(predicate::eq(1))
            .returning(move |_| Ok(access.clone()));

        let request = Request::new(DeleteAccessRequest { id: 1 });

        let api = get_mock_concrete_ecdar_api(mock_services);

        let res = api.delete_access(request).await;

        assert!(res.is_ok());
    }
}
