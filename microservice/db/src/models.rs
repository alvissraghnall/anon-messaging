use crate::unix_timestamp::unix_timestamp;
use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow,
};

#[cargo_with::cargo_as]
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: String, // UUID v7
    pub username: String,
    pub public_key: String, // Base64-encoded SPKI format
    pub public_key_hash: String,
    #[serde(with = "unix_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "unix_timestamp")]
    pub last_login: Option<DateTime<Utc>>,
}

// Create table migration (add to your migrations folder)
/*
-- migrations/0001_create_users.up.sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    public_key TEXT NOT NULL UNIQUE,
    public_key_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP
);
*/
