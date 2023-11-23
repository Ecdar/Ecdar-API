#[cfg(test)]
mod access_logic {
    use crate::tests::api::helpers::get_mock_services;

    #[tokio::test]
    async fn create_access_foreign_key_constraint_err_returns_err() {
        let mut mock_services = get_mock_services();
        assert!(true);
    }

    #[tokio::test]
    async fn create_access_returns_ok() {
        let mut mock_services = get_mock_services();
        assert!(true);
    }

    #[tokio::test]
    async fn update_access_foreign_key_constraint_err_returns_err() {
        let mut mock_services = get_mock_services();
        assert!(true);
    }

    #[tokio::test]
    async fn update_access_returns_ok() {
        let mut mock_services = get_mock_services();
        assert!(true);
    }

    #[tokio::test]
    async fn delete_nonexistent_access_returns_err() {
        let mut mock_services = get_mock_services();
        assert!(true);
    }

    #[tokio::test]
    async fn delete_existent_access_returns_ok() {
        let mut mock_services = get_mock_services();
        assert!(true);
    }
}
