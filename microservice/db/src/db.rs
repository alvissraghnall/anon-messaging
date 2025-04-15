use crate::unix_timestamp::unix_timestamp;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use sqlx::{Error, SqlitePool};
use std::env;
use std::path::PathBuf;
use std::sync::Once;
use uuid::Uuid;

static INIT: Once = Once::new();

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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    // pub id: Option<i64>,
    pub user_id: String,
    pub public_key_hash: String,
    pub encrypted_private_key: String, // base64
    pub encryption_salt: String,       // already a base64 string
    pub encryption_nonce: String,      // base64
}

pub async fn create_db_pool() -> Result<SqlitePool, Error> {
    dotenv().ok();
    let root_dir = PathBuf::from(env::current_dir().unwrap().parent().unwrap());
    let env_file_path = root_dir.join(".env.production");
    INIT.call_once(|| {
        dotenv::from_path(env_file_path).expect("ENV PRODUCTION FILE MUST EXIST!");
    });

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqlitePool::connect(&database_url).await
}

pub async fn insert_user(
    pool: &SqlitePool,
    user_id: &str,
    public_key_hash: &str,
    encrypted_private_key: &str,
    encryption_salt: &str,
    encryption_nonce: &str,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO users (user_id, public_key_hash, encrypted_private_key, encryption_salt, encryption_nonce) 
         VALUES ($1, $2, $3, $4, $5)",
        user_id,
        public_key_hash,
		encrypted_private_key,
        encryption_salt,
        encryption_nonce
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_user_by_id(pool: &SqlitePool, user_id: &str) -> Result<User, Error> {
    let user = sqlx::query_as!(
        User,
        "SELECT user_id, public_key_hash, encrypted_private_key, encryption_salt, encryption_nonce 
         FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub fn generate_user_id() -> String {
    // Generate a UUID and take the first 8 characters
    let uuid = Uuid::new_v4();
    let uuid_hex = uuid.as_simple().to_string();
    uuid_hex[..8].to_string()
}

pub async fn insert_user_with_retry(
    pool: &SqlitePool,
    user_id: &str,
    public_key_hash: &str,
    encrypted_private_key: &str,
    encryption_salt: &str,
    encryption_nonce: &str,
) -> Result<String, String> {
    let mut retries = 0;
    let mut final_user_id = user_id.to_string();

    loop {
        match insert_user(
            pool,
            &final_user_id,
            public_key_hash,
            encrypted_private_key,
            encryption_salt,
            encryption_nonce,
        )
        .await
        {
            Ok(_) => return Ok(final_user_id),
            Err(Error::Database(err)) if err.is_unique_violation() => {
                // If the user_id already exists, generate a new one
                retries += 1;
                if retries > 5 {
                    return Err("Failed to generate a unique user_id after 5 retries".to_string());
                }
                final_user_id = generate_user_id();
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}

/*
pub async fn create_user(
    pool: &SqlitePool,
    custom_user_id: Option<String>,
    public_key: &[u8],
) -> Result<String, String> {
    let public_key_hash = {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        format!("{:x}", hasher.finalize())
    };

    let user_id = match custom_user_id {
        Some(id) => {
            validate_user_id(&id)?;
            id
        }
        None => generate_user_id(),
    };

    insert_user_with_retry(&pool, &user_id, &public_key_hash).await?;
    Ok(user_id)
}
*/

// async fn store_encrypted_message(
//     pool: &SqlitePool,
//     sender_id: &str,
//     recipient_id: &str,
//     encrypted_message: &str,
// ) -> Result<(), String> {
//     sqlx::query!(
//         r#"
//         INSERT INTO encrypted_messages (sender_id, recipient_id, encrypted_message)
//         VALUES ($1, $2, $3)
//         "#,
//         sender_id,
//         recipient_id,
//         encrypted_message
//     )
//     .execute(pool)
//     .await
//     .map_err(|e| match e {
//         sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
//             format!("Sender or recipient does not exist")
//         }
//         _ => format!("Failed to store encrypted message: {}", e),
//     })?;
//
//     Ok(())
// }

async fn fetch_public_key_hash(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> Result<String, sqlx::Error> {
    let public_key_hash = sqlx::query!(
        r#"
        SELECT public_key_hash
        FROM users
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?
    .public_key_hash;

    Ok(public_key_hash)
}

pub async fn create_message(
    pool: &SqlitePool,
    sender_id: &str,
    recipient_id: &str,
    encrypted_content: &str,
    signature: Option<&str>,
    parent_id: Option<i64>,
) -> Result<Option<i64>, Error> {
    let mut conn = pool.acquire().await?;
    let current_time = Utc::now().timestamp();

    let message_id = sqlx::query!(
        r#"
        INSERT INTO messages (sender_id, recipient_id, encrypted_content, signature, parent_id, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
        sender_id,
        recipient_id,
        encrypted_content,
        signature,
        parent_id,
        current_time,
    )
    .fetch_one(pool)
    .await?
    .id;

    Ok(message_id)
}

pub async fn get_message(pool: &SqlitePool, message_id: i64) -> Result<Option<Message>, Error> {
    let raw = sqlx::query_as!(
        RawMessage,
        r#"
        SELECT 
            id, 
            sender_id, 
            recipient_id, 
            encrypted_content, 
            signature, 
            parent_id, 
            is_read, 
            created_at
        FROM messages
        WHERE id = ?
        "#,
        message_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(raw.map(RawMessage::into_message))
}

pub async fn mark_message_read(pool: &SqlitePool, message_id: i64) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE messages
        SET is_read = 1
        WHERE id = ?
        "#,
        message_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_conversation(
    pool: &SqlitePool,
    user1_id: &str,
    user2_id: &str,
    limit: Option<i64>,
) -> Result<Vec<Message>, Error> {
    #[derive(sqlx::FromRow)]
    struct DbMessage {
        id: i64,
        sender_id: String,
        recipient_id: String,
        encrypted_content: String,
        signature: Option<String>,
        parent_id: Option<i64>,
        is_read: i64,
        created_at: i64,
    }

    let db_messages = sqlx::query_as::<_, DbMessage>(
        r#"
        SELECT 
            id,
            sender_id,
            recipient_id,
            encrypted_content,
            signature,
            parent_id,
            is_read,
            created_at
        FROM messages
        WHERE (sender_id = ? AND recipient_id = ?)
           OR (sender_id = ? AND recipient_id = ?)
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(user1_id)
    .bind(user2_id)
    .bind(user2_id)
    .bind(user1_id)
    .bind(limit.unwrap_or(100))
    .fetch_all(pool)
    .await?;

    let messages = db_messages
        .into_iter()
        .map(|db_msg| Message {
            id: db_msg.id,
            sender_id: db_msg.sender_id,
            recipient_id: db_msg.recipient_id,
            encrypted_content: db_msg.encrypted_content,
            signature: db_msg.signature,
            parent_id: db_msg.parent_id,
            is_read: db_msg.is_read != 0,
            created_at: DateTime::from_timestamp(db_msg.created_at, 0)
                .unwrap_or_else(|| Utc::now()),
        })
        .collect();

    Ok(messages)
}

pub async fn get_unread_messages(pool: &SqlitePool, user_id: &str) -> Result<Vec<Message>, Error> {
    #[derive(sqlx::FromRow)]
    struct DbMessage {
        id: i64,
        sender_id: String,
        recipient_id: String,
        encrypted_content: String,
        signature: Option<String>,
        parent_id: Option<i64>,
        is_read: i64,
        created_at: i64,
    }

    let unread_messages = sqlx::query_as::<_, DbMessage>(
        r#"
        SELECT id, sender_id, recipient_id, encrypted_content, signature, parent_id, created_at,
        is_read
        FROM messages
        WHERE recipient_id = ? AND is_read = 0
        ORDER BY created_at ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let messages = unread_messages
        .into_iter()
        .map(|db_msg| Message {
            id: db_msg.id,
            sender_id: db_msg.sender_id,
            recipient_id: db_msg.recipient_id,
            encrypted_content: db_msg.encrypted_content,
            signature: db_msg.signature,
            parent_id: db_msg.parent_id,
            is_read: db_msg.is_read != 0,
            created_at: DateTime::from_timestamp(db_msg.created_at, 0)
                .unwrap_or_else(|| Utc::now()),
        })
        .collect();

    Ok(messages)
}

pub async fn get_thread_replies(
    pool: &SqlitePool,
    parent_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Message>, Error> {
    let raw_messages = sqlx::query_as::<_, RawMessage>(
        r#"
        SELECT id, sender_id, recipient_id, encrypted_content, signature, parent_id, created_at, is_read
        FROM messages
        WHERE parent_id = ?
        ORDER BY created_at ASC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(parent_id)
    .bind(limit.unwrap_or(100))
    .bind(offset.unwrap_or(0))
    .fetch_all(pool)
    .await?;

    Ok(raw_messages
        .into_iter()
        .map(RawMessage::into_message)
        .collect())
}

/// Gets a complete thread including the parent message and all replies
pub async fn get_complete_thread(
    pool: &SqlitePool,
    thread_root_id: i64,
    limit: Option<i64>,
) -> Result<Vec<Message>, Error> {
    let parent_message = match get_message(pool, thread_root_id).await? {
        Some(msg) => msg,
        None => return Err(Error::RowNotFound),
    };

    // get all replies
    let mut replies = get_thread_replies(pool, thread_root_id, limit, None).await?;

    // Combine into single thread (parent first, then replies)
    let mut thread = Vec::with_capacity(1 + replies.len());
    thread.push(parent_message);
    thread.append(&mut replies);

    Ok(thread)
}

pub async fn get_user_threads(
    pool: &SqlitePool,
    user_id: &str,
    limit: Option<i64>,
) -> Result<Vec<Message>, Error> {
    let raw_messages = sqlx::query_as::<_, RawMessage>(
        r#"
        SELECT DISTINCT m.id, m.sender_id, m.recipient_id, m.encrypted_content, 
                m.signature, m.parent_id, m.created_at, m.is_read
        FROM messages m
        JOIN messages r ON m.id = r.parent_id
        WHERE m.sender_id = ? OR m.recipient_id = ?
        ORDER BY m.created_at DESC
        LIMIT ?
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .bind(limit.unwrap_or(20))
    .fetch_all(pool)
    .await?;

    Ok(raw_messages
        .into_iter()
        .map(RawMessage::into_message)
        .collect())
}

pub async fn store_refresh_token(
    db: &SqlitePool,
    id: &str,
    user_id: &str,
    token_hash: &str,
    expires_at: i64,
    device_info: Option<&str>,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens 
        (id, user_id, token_hash, expires_at, device_info)
        VALUES (?, ?, ?, datetime(?, 'unixepoch'), ?)
        "#,
        id,
        user_id,
        token_hash,
        expires_at,
        device_info
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn validate_refresh_token(
    db: &SqlitePool,
    user_id: &str,
    token_hash: &str,
) -> Result<bool, Error> {
    let record = sqlx::query!(
        r#"
        SELECT id FROM refresh_tokens
        WHERE user_id = ? AND token_hash = ? AND expires_at > CURRENT_TIMESTAMP
        "#,
        user_id,
        token_hash
    )
    .fetch_optional(db)
    .await?;

    Ok(record.is_some())
}

pub async fn revoke_refresh_token(
    db: &SqlitePool,
    token_hash: &str,
    reason: Option<&str>,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO revoked_tokens (token_hash, reason)
        SELECT token_hash, ? FROM refresh_tokens
        WHERE token_hash = ?
        "#,
        reason,
        token_hash
    )
    .execute(db)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM refresh_tokens
        WHERE token_hash = ?
        "#,
        token_hash
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn cleanup_expired_tokens(db: &SqlitePool) -> Result<u64, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM refresh_tokens
        WHERE expires_at <= CURRENT_TIMESTAMP
        "#
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
#[path = "db.test.rs"]
mod tests;
