use async_trait::async_trait;
use db::{Error as SqlxError, SqlitePool, db as database, uuid::Uuid};
use mockall::automock;
use shared::errors::AppError;

#[automock]
#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn store_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: i64,
        device_info: Option<String>,
    ) -> Result<(), AppError>;

    async fn validate_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
    ) -> Result<bool, AppError>;

    async fn revoke_refresh_token(
        &self,
        token_hash: &str,
        reason: Option<String>,
    ) -> Result<(), AppError>;
}

#[async_trait]
impl TokenRepository for SqlitePool {
    async fn store_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: i64,
        device_info: Option<String>,
    ) -> Result<(), AppError> {
        Ok(database::store_refresh_token(
            self,
            user_id,
            token_hash,
            expires_at,
            device_info.as_deref(),
        )
        .await?)
    }

    async fn validate_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
    ) -> Result<bool, AppError> {
        Ok(database::validate_refresh_token(self, user_id, token_hash).await?)
    }

    async fn revoke_refresh_token(
        &self,
        token_hash: &str,
        reason: Option<String>,
    ) -> Result<(), AppError> {
        Ok(database::revoke_refresh_token(self, token_hash, reason.as_deref()).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::uuid::Uuid;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_store_refresh_token_success() {
        let mut mock = MockTokenRepository::new();
        let uid = Uuid::now_v7();

        mock.expect_store_refresh_token()
            .with(
                eq(uid.clone()),
                eq("hashed_token"),
                eq(1234567890),
                eq(Some("iPhone 13".to_string())),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let result = mock
            .store_refresh_token(
                uid,
                "hashed_token",
                1234567890,
                Some("iPhone 13".to_string()),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_refresh_token_valid() {
        let mut mock = MockTokenRepository::new();
        let uid = Uuid::now_v7();

        mock.expect_validate_refresh_token()
            .with(eq(uid.clone()), eq("valid_hash"))
            .returning(|_, _| Ok(true));

        let result = mock.validate_refresh_token(uid, "valid_hash").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_revoke_refresh_token_with_reason() {
        let mut mock = MockTokenRepository::new();
        mock.expect_revoke_refresh_token()
            .with(eq("revoke_me"), eq(Some("session_terminated".to_string())))
            .returning(|_, _| Ok(()));

        let result = mock
            .revoke_refresh_token("revoke_me", Some("session_terminated".to_string()))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_refresh_token_invalid() {
        let mut mock = MockTokenRepository::new();
        let uid = Uuid::now_v7();

        mock.expect_validate_refresh_token()
            .with(eq(uid.clone()), eq("invalid_hash"))
            .returning(|_, _| Ok(false));

        let result = mock.validate_refresh_token(uid, "invalid_hash").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_revoke_refresh_token_without_reason() {
        let mut mock = MockTokenRepository::new();
        mock.expect_revoke_refresh_token()
            .with(eq("revoke_me"), eq(None))
            .returning(|_, _| Ok(()));

        let result = mock.revoke_refresh_token("revoke_me", None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_refresh_token_error() {
        let mut mock = MockTokenRepository::new();
        let uid = Uuid::now_v7();

        mock.expect_store_refresh_token()
            .with(eq(uid.clone()), eq("error_token"), eq(1234567890), eq(None))
            .times(1)
            .returning(|_, _, _, _| {
                Err(AppError::DatabaseError(db::Error::InvalidArgument(
                    "Failed to store token".to_string(),
                )))
            });

        let result = mock
            .store_refresh_token(uid, "error_token", 1234567890, None)
            .await;

        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                AppError::DatabaseError(err) => match err {
                    db::Error::InvalidArgument(msg) => assert_eq!(msg, "Failed to store token"),
                    _ => todo!(),
                },
                _ => panic!("Expected DatabaseError, got different error type"),
            }
        }
    }

    #[tokio::test]
    async fn test_validate_refresh_token_error() {
        let mut mock = MockTokenRepository::new();
        let uid = Uuid::now_v7();

        mock.expect_validate_refresh_token()
            .with(eq(uid.clone()), eq("error_hash"))
            .returning(|_, _| {
                Err(AppError::DatabaseError(db::Error::InvalidArgument(
                    "Failed to store token".to_string(),
                )))
            });

        let result = mock.validate_refresh_token(uid, "error_hash").await;

        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                AppError::DatabaseError(err) => match err {
                    db::Error::InvalidArgument(msg) => assert_eq!(msg, "Failed to store token"),
                    _ => todo!(),
                },
                _ => panic!("Expected DatabaseError, got different error type"),
            }
        }
    }

    #[tokio::test]
    async fn test_revoke_refresh_token_error() {
        let mut mock = MockTokenRepository::new();
        mock.expect_revoke_refresh_token()
            .with(eq("error_token"), eq(Some("forced_error".to_string())))
            .returning(|_, _| {
                Err(AppError::DatabaseError(db::Error::InvalidArgument(
                    "Failed to store token".to_string(),
                )))
            });

        let result = mock
            .revoke_refresh_token("error_token", Some("forced_error".to_string()))
            .await;

        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                AppError::DatabaseError(err) => match err {
                    db::Error::InvalidArgument(msg) => assert_eq!(msg, "Failed to store token"),
                    _ => todo!(),
                },
                _ => panic!("Expected DatabaseError, got different error type"),
            }
        }
    }

    #[tokio::test]
    async fn test_store_refresh_token_minimal() {
        let mut mock = MockTokenRepository::new();
        let uid = Uuid::now_v7();

        mock.expect_store_refresh_token()
            .with(
                eq(uid.clone()),
                eq("simple_token"),
                eq(1234567890),
                eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let result = mock
            .store_refresh_token(uid, "simple_token", 1234567890, None)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_token_operations() {
        let mut mock = MockTokenRepository::new();
        let uid = Uuid::now_v7();
        let token_hash = "multi_op_token";

        //store a token
        mock.expect_store_refresh_token()
            .with(eq(uid.clone()), eq(token_hash), eq(1234567890), eq(None))
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        mock.expect_validate_refresh_token()
            .with(eq(uid.clone()), eq(token_hash))
            .times(1)
            .returning(|_, _| Ok(true));

        mock.expect_revoke_refresh_token()
            .with(eq(token_hash), eq(Some("logout".to_string())))
            .times(1)
            .returning(|_, _| Ok(()));

        // Execute the sequence
        let store_result = mock
            .store_refresh_token(uid.clone(), token_hash, 1234567890, None)
            .await;
        assert!(store_result.is_ok());

        let validate_result = mock.validate_refresh_token(uid, token_hash).await;
        assert!(validate_result.is_ok());
        assert_eq!(validate_result.unwrap(), true);

        let revoke_result = mock
            .revoke_refresh_token(token_hash, Some("logout".to_string()))
            .await;
        assert!(revoke_result.is_ok());
    }
}
