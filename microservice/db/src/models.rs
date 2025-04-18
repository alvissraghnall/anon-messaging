use crate::unix_timestamp::unix_timestamp;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, TimestampSecondsWithFrac};
use sqlx::{
    types::{
        chrono::{DateTime, NaiveDateTime, Utc},
        Uuid,
    },
    FromRow,
};

#[serde_as]
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    #[serde(with = "uuid::serde::simple")]
    pub id: Uuid,
    pub username: String,
    pub public_key: String, // Base64-encoded SPKI format
    pub public_key_hash: String,
    #[serde_as(as = "TimestampSecondsWithFrac<String>")]
    pub created_at: NaiveDateTime,
    #[serde_as(as = "Option<TimestampSecondsWithFrac<String>>")]
    pub last_login: Option<NaiveDateTime>,
    #[serde_as(as = "TimestampSecondsWithFrac<String>")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RawMessage {
    pub id: i64,
    pub sender_id: String,
    pub recipient_id: String,
    pub encrypted_content: String,
    pub signature: Option<String>,
    pub parent_id: Option<i64>,
    pub created_at: i64,
    pub is_read: i64,
}

impl RawMessage {
    pub fn into_message(self) -> Message {
        Message {
            id: self.id,
            sender_id: self.sender_id,
            recipient_id: self.recipient_id,
            encrypted_content: self.encrypted_content,
            signature: self.signature,
            parent_id: self.parent_id,
            created_at: DateTime::from_timestamp(self.created_at, 0).unwrap_or_else(|| Utc::now()),
            is_read: self.is_read != 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: i64,
    pub sender_id: String,
    pub recipient_id: String,
    pub encrypted_content: String,
    pub parent_id: Option<i64>,
    pub signature: Option<String>,
    #[serde(with = "unix_timestamp")]
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
}
