#[cfg(test)]
mod auth {
    use crate::api::auth::{RequestExt, Token, TokenError, TokenType};
    use std::{env, str::FromStr};
    use tonic::{metadata::MetadataValue, Request};

    #[tokio::test]
    async fn request_token_trims_bearer() {
        let token = "Bearer 1234567890";
        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert("authorization", MetadataValue::from_str(token).unwrap());

        let result = request.token_str().unwrap().unwrap();

        assert_eq!(result, token.trim_start_matches("Bearer "));
    }

    #[tokio::test]
    async fn request_token_no_token_returns_none() {
        let request = Request::new(());
        let result = request.token_str().unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn token_new_access_returns_token() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let uid = "1";
        let result = Token::new(TokenType::AccessToken, uid);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn token_new_refresh_returns_token() {
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let uid = "1";
        let result = Token::new(TokenType::RefreshToken, uid);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_token_valid_access_returns_tokendata() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = Token::new(TokenType::AccessToken, "1").unwrap();
        let result = token.validate();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_token_valid_refresh_returns_tokendata() {
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let token = Token::new(TokenType::RefreshToken, "1").unwrap();
        let result = token.validate();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_token_invalid_returns_err() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let result_access = Token::from_str(TokenType::AccessToken, "invalid_token").validate();
        let result_refresh = Token::from_str(TokenType::RefreshToken, "invalid_token").validate();

        assert_eq!(result_access.unwrap_err(), TokenError::InvalidToken);
        assert_eq!(result_refresh.unwrap_err(), TokenError::InvalidToken);
    }

    #[tokio::test]
    async fn token_type_access_returns_access() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = Token::new(TokenType::AccessToken, "1").unwrap();
        let result = token.token_type();

        assert_eq!(result, TokenType::AccessToken);
    }

    #[tokio::test]
    async fn token_type_refresh_returns_refresh() {
        env::set_var("REFRESH_TOKEN_HS512_SECRET", "refresh_secret");

        let token = Token::new(TokenType::RefreshToken, "1").unwrap();
        let result = token.token_type();

        assert_eq!(result, TokenType::RefreshToken);
    }

    #[tokio::test]
    async fn token_to_string_returns_string() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = Token::new(TokenType::AccessToken, "1").unwrap();
        let result = token.to_string();

        assert_eq!(result, token.as_str());
    }

    #[tokio::test]
    async fn token_as_str_returns_string() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = Token::new(TokenType::AccessToken, "1").unwrap();
        let result = token.as_str();

        assert_eq!(result, token.to_string());
    }

    #[tokio::test]
    async fn token_from_str_returns_token() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = Token::new(TokenType::AccessToken, "1").unwrap();
        let token_from_str = Token::from_str(TokenType::AccessToken, token.as_str());

        let result = token_from_str.validate();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn token_from_str_invalid_returns_err() {
        env::set_var("ACCESS_TOKEN_HS512_SECRET", "access_secret");

        let token = Token::from_str(TokenType::AccessToken, "invalid_token");
        let result = token.validate();

        assert!(result.is_err());
    }
}
