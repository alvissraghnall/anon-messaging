use crate::models::Message;
use async_trait::async_trait;
use sqlx::Error;
use uuid::Uuid;
use crate::db::{self, SqliteDb};
use mockall::{automock};

#[automock]
#[async_trait]
pub trait MessageDb {
    async fn create_message(
        &self,
        sender_id: Uuid,
        recipient_id: Uuid,
        encrypted_content: &str,
        signature: Option<String>,
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
}

#[async_trait]
impl MessageDb for SqliteDb {
    async fn create_message(
        &self,
        sender_id: Uuid,
        recipient_id: Uuid,
        encrypted_content: &str,
        signature: Option<String>,
        parent_id: Option<i64>,
    ) -> Result<Option<i64>, Error> {
        db::create_message(&self.pool, sender_id, recipient_id, encrypted_content, signature.as_deref(), parent_id).await
    }

    async fn get_message(&self, message_id: i64) -> Result<Option<Message>, Error> {
        db::get_message(&self.pool, message_id).await
    }

    async fn mark_message_read(&self, message_id: i64) -> Result<(), Error> {
        db::mark_message_read(&self.pool, message_id).await
    }

    async fn get_conversation(
        &self,
        user1_id: Uuid,
        user2_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, Error> {
        db::get_conversation(&self.pool, user1_id, user2_id, limit).await
    }

    async fn get_unread_messages(&self, user_id: Uuid) -> Result<Vec<Message>, Error> {
        db::get_unread_messages(&self.pool, user_id).await
    }

    async fn get_thread_replies(
        &self,
        parent_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Message>, Error> {
        db::get_thread_replies(&self.pool, parent_id, limit, offset).await
    }

    async fn get_complete_thread(
        &self,
        thread_root_id: i64,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, Error> {
        db::get_complete_thread(&self.pool, thread_root_id, limit).await
    }

    async fn get_user_threads(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, Error> {
        db::get_user_threads(&self.pool, user_id, limit).await
    }
}
