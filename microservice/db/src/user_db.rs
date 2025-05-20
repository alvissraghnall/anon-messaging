use crate::{models::User, public_key::PublicKey, public_key_hash::PublicKeyHash};
use async_trait::async_trait;
use sqlx::Error;
use uuid::Uuid;
use crate::db::{self, SqliteDb};
use mockall::{automock};

#[automock]
#[async_trait]
pub trait UserDb {
    async fn insert_user(
        &self,
        public_key_hash: &PublicKeyHash,
        public_key: &PublicKey,
        username: &str,
    ) -> Result<Uuid, Error>;

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, Error>;

    async fn get_user_by_pubkey(&self, pubkey_hash: &PublicKeyHash) -> Result<User, Error>;

    async fn get_users(&self, limit: Option<i64>) -> Result<Vec<User>, Error>;

    async fn update_user<'a>(
        &self,
        user_id: Uuid,
        new_username: Option<&'a str>,
        new_public_key: Option<&'a PublicKey>,
        new_public_key_hash: Option<&'a PublicKeyHash>,
    ) -> Result<(), Error>;

    async fn fetch_public_key_hash(&self, user_id: Uuid) -> Result<String, Error>;
}

#[async_trait]
impl UserDb for SqliteDb {
    async fn insert_user(
        &self,
        public_key_hash: &PublicKeyHash,
        public_key: &PublicKey,
        username: &str,
    ) -> Result<Uuid, Error> {
        db::insert_user(&self.pool, public_key_hash, public_key, username).await
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, Error> {
        db::get_user_by_id(&self.pool, user_id).await
    }

    async fn get_user_by_pubkey(&self, pubkey_hash: &PublicKeyHash) -> Result<User, Error> {
        db::get_user_by_pubkey(&self.pool, pubkey_hash).await
    }

    async fn get_users(&self, limit: Option<i64>) -> Result<Vec<User>, Error> {
        db::get_users(&self.pool, limit).await
    }

    async fn update_user<'a>(
        &self,
        user_id: Uuid,
        new_username: Option<&'a str>,
        new_public_key: Option<&'a PublicKey>,
        new_public_key_hash: Option<&'a PublicKeyHash>,
    ) -> Result<(), Error> {
        db::update_user(&self.pool, user_id, new_username, new_public_key, new_public_key_hash).await
    }

    async fn fetch_public_key_hash(&self, user_id: Uuid) -> Result<String, Error> {
        db::fetch_public_key_hash(&self.pool, user_id).await
    }
}
