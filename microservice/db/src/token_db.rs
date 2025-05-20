use async_trait::async_trait;
use sqlx::Error;
use uuid::Uuid;
use crate::db::{self, SqliteDb};
use mockall::{automock};

#[automock]
#[async_trait]
pub trait TokenDb {
    async fn store_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: i64,
        device_info: Option<String>,
    ) -> Result<(), Error>;

    async fn validate_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
    ) -> Result<bool, Error>;

    async fn revoke_refresh_token(
        &self,
        token_hash: &str,
        reason: Option<String>,
    ) -> Result<(), Error>;

    async fn cleanup_expired_tokens(&self) -> Result<u64, Error>;
}

#[async_trait]
impl TokenDb for SqliteDb {
    async fn store_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: i64,
        device_info: Option<String>,
    ) -> Result<(), Error> {
        db::store_refresh_token(&self.pool, user_id, token_hash, expires_at, device_info.as_deref()).await
    }

    async fn validate_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
    ) -> Result<bool, Error> {
        db::validate_refresh_token(&self.pool, user_id, token_hash).await
    }

    async fn revoke_refresh_token(
        &self,
        token_hash: &str,
        reason: Option<String>,
    ) -> Result<(), Error> {
        db::revoke_refresh_token(&self.pool, token_hash, reason.as_deref()).await
    }

    async fn cleanup_expired_tokens(&self) -> Result<u64, Error> {
        db::cleanup_expired_tokens(&self.pool).await
    }
}
