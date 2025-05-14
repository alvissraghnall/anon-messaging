use crate::models::{Message, RawMessage, User};
use crate::{public_key::PublicKey, public_key_hash::PublicKeyHash};
use async_trait::async_trait;
use sqlx::{Error, SqlitePool};
use uuid::Uuid;

#[async_trait]
pub trait Db {
    async fn insert_user(
        &self,
        public_key_hash: &PublicKeyHash,
        public_key: &PublicKey,
        username: &str,
    ) -> Result<Uuid, Error>;

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, Error>;

    async fn get_user_by_pubkey(&self, pubkey_hash: &PublicKeyHash) -> Result<User, Error>;

    async fn get_users(&self, limit: Option<i64>) -> Result<Vec<User>, Error>;

    async fn create_message(
        &self,
        sender_id: Uuid,
        recipient_id: Uuid,
        encrypted_content: &str,
        signature: Option<&str>,
        parent_id: Option<i64>,
    ) -> Result<Option<i64>, Error>;

    async fn get_message(&self, message_id: i64) -> Result<Option<Message>, Error>;

    async fn mark_message_read(&self, message_id: i64) -> Result<(), Error>;

    async fn get_conversation(
        &self,
        user1_id: Uuid,
        user2_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, Error>;

    async fn get_unread_messages(&self, user_id: Uuid) -> Result<Vec<Message>, Error>;

    async fn get_thread_replies(
        &self,
        parent_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Message>, Error>;

    async fn get_complete_thread(
        &self,
        thread_root_id: i64,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, Error>;

    async fn get_user_threads(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, Error>;

    async fn store_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: i64,
        device_info: Option<&str>,
    ) -> Result<(), Error>;

    async fn validate_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
    ) -> Result<bool, Error>;

    async fn revoke_refresh_token(
        &self,
        token_hash: &str,
        reason: Option<&str>,
    ) -> Result<(), Error>;

    async fn cleanup_expired_tokens(&self) -> Result<u64, Error>;

    async fn fetch_public_key_hash(&self, user_id: Uuid) -> Result<String, Error>;

    async fn update_user(
        &self,
        user_id: Uuid,
        new_username: Option<&str>,
        new_public_key: Option<&PublicKey>,
        new_public_key_hash: Option<&PublicKeyHash>,
    ) -> Result<(), Error>;
}
