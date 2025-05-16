use db::uuid::Uuid;
use shared::errors::AppError;

use super::repository::TokenRepository;

#[derive(Clone)]
pub struct TokenService<R: TokenRepository> {
    repository: R,
}

impl<R: TokenRepository> TokenService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn store_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: i64,
        device_info: Option<String>,
    ) -> Result<(), AppError> {
        self.repository
            .store_refresh_token(user_id, token_hash, expires_at, device_info)
            .await
    }

    pub async fn validate_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
    ) -> Result<bool, AppError> {
        self.repository
            .validate_refresh_token(user_id, token_hash)
            .await
    }

    pub async fn revoke_refresh_token(
        &self,
        token_hash: &str,
        reason: Option<String>,
    ) -> Result<(), AppError> {
        self.repository
            .revoke_refresh_token(token_hash, reason)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::repository::MockTokenRepository;
    use db::Error as SqlxError;
    use mockall::predicate::*;

    impl Clone for MockTokenRepository {
        fn clone(&self) -> Self {
            Self::default()
        }
    }

    #[tokio::test]
    async fn test_service_store_token_propagates_error() {
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_store_refresh_token()
            .returning(|_, _, _, _| Err(AppError::DatabaseError(SqlxError::PoolClosed)));

        let service = TokenService::new(mock_repo);
        let result = service
            .store_refresh_token(Uuid::now_v7(), "bad_token", 0, None)
            .await;

        assert!(matches!(result, Err(AppError::DatabaseError(_))));
    }

    #[tokio::test]
    async fn test_service_validate_token_checks_repository() {
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_validate_refresh_token()
            .with(
                eq(Uuid::parse_str("d1b6a896-7f15-4a5e-9f1a-3a4a2a7a8a9a").unwrap()),
                eq("correct_hash"),
            )
            .returning(|_, _| Ok(true));

        let service = TokenService::new(mock_repo);
        let user_id = Uuid::parse_str("d1b6a896-7f15-4a5e-9f1a-3a4a2a7a8a9a").unwrap();
        let result = service
            .validate_refresh_token(user_id, "correct_hash")
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_service_store_token_success() {
        let mut mock_repo = MockTokenRepository::new();
        let user_id = Uuid::now_v7();
        let token_hash = "valid_token_hash";
        let expires_at = 1234567890;
        let device_info = Some("Android Pixel 6".to_string());

        mock_repo
            .expect_store_refresh_token()
            .with(
                eq(user_id),
                eq(token_hash),
                eq(expires_at),
                eq(device_info.clone()),
            )
            .returning(|_, _, _, _| Ok(()));

        let service = TokenService::new(mock_repo);
        let result = service
            .store_refresh_token(user_id, token_hash, expires_at, device_info)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_validate_token_invalid() {
        let mut mock_repo = MockTokenRepository::new();
        let user_id = Uuid::now_v7();
        let token_hash = "invalid_token";

        mock_repo
            .expect_validate_refresh_token()
            .with(eq(user_id), eq(token_hash))
            .returning(|_, _| Ok(false));

        let service = TokenService::new(mock_repo);
        let result = service.validate_refresh_token(user_id, token_hash).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_service_validate_token_propagates_error() {
        let mut mock_repo = MockTokenRepository::new();
        let user_id = Uuid::now_v7();
        let token_hash = "error_token";

        mock_repo
            .expect_validate_refresh_token()
            .with(eq(user_id), eq(token_hash))
            .returning(|_, _| {
                Err(AppError::DatabaseError(db::Error::InvalidArgument(
                    "Validation error".to_string(),
                )))
            });

        let service = TokenService::new(mock_repo);
        let result = service.validate_refresh_token(user_id, token_hash).await;

        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                AppError::DatabaseError(err) => match err {
                    db::Error::InvalidArgument(msg) => assert_eq!(msg, "Validation error"),
                    _ => todo!(),
                },
                _ => panic!("Expected DatabaseError, got different error type"),
            }
        }
    }

    #[tokio::test]
    async fn test_service_revoke_token_success() {
        let mut mock_repo = MockTokenRepository::new();
        let token_hash = "revoke_this_token";
        let reason = Some("user_logout".to_string());

        mock_repo
            .expect_revoke_refresh_token()
            .with(eq(token_hash), eq(reason.clone()))
            .returning(|_, _| Ok(()));

        let service = TokenService::new(mock_repo);
        let result = service.revoke_refresh_token(token_hash, reason).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_revoke_token_without_reason() {
        let mut mock_repo = MockTokenRepository::new();
        let token_hash = "revoke_this_token";

        mock_repo
            .expect_revoke_refresh_token()
            .with(eq(token_hash), eq(None))
            .returning(|_, _| Ok(()));

        let service = TokenService::new(mock_repo);
        let result = service.revoke_refresh_token(token_hash, None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_revoke_token_propagates_error() {
        let mut mock_repo = MockTokenRepository::new();
        let token_hash = "error_token";
        let reason = Some("forced_error".to_string());

        mock_repo
            .expect_revoke_refresh_token()
            .with(eq(token_hash), eq(reason.clone()))
            .returning(|_, _| {
                Err(AppError::DatabaseError(db::Error::InvalidArgument(
                    "Revocation failed".to_string(),
                )))
            });

        let service = TokenService::new(mock_repo);
        let result = service.revoke_refresh_token(token_hash, reason).await;

        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                AppError::DatabaseError(err) => match err {
                    db::Error::InvalidArgument(msg) => assert_eq!(msg, "Revocation failed"),
                    _ => todo!(),
                },
                _ => panic!("Expected DatabaseError, got different error type"),
            }
        }
    }

    #[tokio::test]
    async fn test_service_clone() {
        let mock_repo = MockTokenRepository::new();
        let service = TokenService::new(mock_repo);

        let _cloned = service.clone();
    }

    #[tokio::test]
    async fn test_service_token_lifecycle() {
        let mut mock_repo = MockTokenRepository::new();
        let user_id = Uuid::now_v7();
        let token_hash = "lifecycle_token";
        let expires_at = 1234567890;

        mock_repo
            .expect_store_refresh_token()
            .with(eq(user_id), eq(token_hash), eq(expires_at), eq(None))
            .returning(|_, _, _, _| Ok(()));

        mock_repo
            .expect_validate_refresh_token()
            .with(eq(user_id), eq(token_hash))
            .returning(|_, _| Ok(true));

        mock_repo
            .expect_revoke_refresh_token()
            .with(eq(token_hash), eq(Some("session_ended".to_string())))
            .returning(|_, _| Ok(()));

        let service = TokenService::new(mock_repo);

        // Store token
        let store_result = service
            .store_refresh_token(user_id, token_hash, expires_at, None)
            .await;
        assert!(store_result.is_ok());

        let validate_result = service.validate_refresh_token(user_id, token_hash).await;
        assert!(validate_result.is_ok());
        assert_eq!(validate_result.unwrap(), true);

        let revoke_result = service
            .revoke_refresh_token(token_hash, Some("session_ended".to_string()))
            .await;
        assert!(revoke_result.is_ok());
    }
}
