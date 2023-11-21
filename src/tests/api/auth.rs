#[cfg(test)]
mod auth {
    use crate::api::auth;
    use crate::api::auth::TokenType;
    use std::{env, str::FromStr};
    use tonic::{metadata::MetadataValue, Request};

    #[tokio::test]
    async fn gtfr_bearer_token_trims_token() {
        let token = "Bearer 1234567890";
        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert("authorization", MetadataValue::from_str(token).unwrap());

        let result = auth::get_token_from_request(&request).unwrap();

        assert_eq!(result, token.trim_start_matches("Bearer "));
    }

    #[tokio::test]
    async fn gtfr_no_token_returns_err() {
        let request = Request::new(());

        let result = auth::get_token_from_request(&request);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_token_access_returns_token() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let uid = "1";
        let result = auth::create_token(auth::TokenType::AccessToken, uid);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_token_refresh_returns_token() {
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let uid = "1";
        let result = auth::create_token(auth::TokenType::RefreshToken, uid);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_token_valid_access_returns_tokendata() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = auth::create_token(auth::TokenType::AccessToken, "1").unwrap();
        let result = auth::validate_token(token, TokenType::AccessToken);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_token_valid_refresh_returns_tokendata() {
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let token = auth::create_token(auth::TokenType::RefreshToken, "1").unwrap();
        let result = auth::validate_token(token, TokenType::RefreshToken);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_token_invalid_returns_err() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let result_access =
            auth::validate_token("invalid_token".to_string(), TokenType::AccessToken);
        let result_refresh =
            auth::validate_token("invalid_token".to_string(), TokenType::RefreshToken);
        assert!(result_access.is_err() && result_refresh.is_err());
    }

    #[tokio::test]
    async fn validate_token_wrong_signature_returns_err() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let token = auth::create_token(auth::TokenType::AccessToken, "1").unwrap();
        let result_access = auth::validate_token(token, TokenType::RefreshToken);

        let token = auth::create_token(auth::TokenType::RefreshToken, "1").unwrap();
        let result_refresh = auth::validate_token(token, TokenType::AccessToken);

        assert!(result_access.is_err() && result_refresh.is_err());
    }
}
