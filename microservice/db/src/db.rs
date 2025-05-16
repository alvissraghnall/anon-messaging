use crate::models::{Message, RawMessage, User};
use crate::{public_key::PublicKey, public_key_hash::PublicKeyHash};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use sqlx::{sqlite::SqliteArguments, Arguments, Row};
use sqlx::{Error, SqlitePool};
use std::env;
use std::path::PathBuf;
use std::sync::Once;
use uuid::Uuid;

use futures::future::BoxFuture;
use std::cmp::Ordering;
use std::collections::BTreeMap;

static INIT: Once = Once::new();

#[derive(Clone)]
pub struct SqliteDb {
    pub pool: SqlitePool,
}

impl SqliteDb {
    pub fn new(pool: SqlitePool) -> Self {
        SqliteDb { pool }
    }
}

pub async fn create_db_pool() -> Result<SqlitePool, Error> {
    dotenv().ok();
    let env_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env.production");
    INIT.call_once(|| {
        dotenv::from_path(env_file_path).expect("ENV PRODUCTION FILE MUST EXIST!");
    });

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqlitePool::connect(&database_url).await
}

pub async fn insert_user(
    pool: &SqlitePool,
    public_key_hash: &PublicKeyHash,
    public_key: &PublicKey,
    username: &str,
) -> Result<Uuid, Error> {
    let id = Uuid::now_v7();
    let id_str = id.to_string();
    println!("{} {} {}", id, id.to_string().len(), id.as_bytes().len());
    println!(
        "{} {} {}",
        public_key_hash.as_str(),
        public_key_hash.as_str().to_string().len(),
        public_key_hash.as_str().as_bytes().len()
    );

    let mut args = SqliteArguments::default();

    args.add(id);
    args.add(public_key);
    args.add(public_key_hash);
    args.add(username);

    sqlx::query_with(
        r#"INSERT INTO users (id, public_key, public_key_hash, username)
         VALUES ($1, $2, $3, $4)
         "#,
        args,
    )
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn get_user_by_id(pool: &SqlitePool, user_id: Uuid) -> Result<User, Error> {
    let mut args = SqliteArguments::default();

    args.add(user_id);

    let user = sqlx::query_as_with::<_, User, _>(
        r#"SELECT 
            id,
            public_key, 
            public_key_hash, 
            username, 
            created_at, 
            last_login, 
            updated_at
         FROM users WHERE id = $1"#,
        args,
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn get_user_by_pubkey(
    pool: &SqlitePool,
    pubkey_hash: &PublicKeyHash,
) -> Result<User, Error> {
    let mut args = SqliteArguments::default();

    args.add(pubkey_hash);

    let user = sqlx::query_as_with::<_, User, _>(
        r#"SELECT 
            id, 
            public_key, 
            public_key_hash, 
            username, 
            created_at, 
            last_login, 
            updated_at
         FROM users WHERE public_key_hash = $1"#,
        args,
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn get_users(pool: &SqlitePool, limit: Option<i64>) -> Result<Vec<User>, Error> {
    let users = sqlx::query_as::<_, User>(
        r#"
            SELECT id,
                    public_key_hash,
                    public_key,
                    username,
                    created_at,
                    last_login,
                    updated_at
            FROM users
            LIMIT $1
        "#,
    )
    .bind(limit.unwrap_or(1000))
    .fetch_all(pool)
    .await?;

    Ok(users)
}

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

pub async fn fetch_public_key_hash(
    pool: &sqlx::SqlitePool,
    user_id: Uuid,
) -> Result<String, sqlx::Error> {
    let public_key_hash = sqlx::query(
        r#"
        SELECT public_key_hash
        FROM users
        WHERE id = ?
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?
    .try_get("public_key_hash")?;

    Ok(public_key_hash)
}

pub async fn update_user(
    pool: &SqlitePool,
    user_id: Uuid,
    new_username: Option<&str>,
    new_public_key: Option<&PublicKey>,
    new_public_key_hash: Option<&PublicKeyHash>,
) -> Result<(), Error> {
    let current_user = get_user_by_id(&pool, user_id).await?;

    if let Some(username) = new_username {
        if username.is_empty() {
            return Err(Error::InvalidArgument(
                "Username cannot be empty".to_string(),
            ));
        }
    }

    if let Some(public_key) = new_public_key {
        if public_key.as_str().is_empty() {
            return Err(Error::InvalidArgument(
                "Public Key cannot be empty".to_string(),
            ));
        }
    }

    if let Some(public_key_hash) = new_public_key_hash {
        if public_key_hash.as_str().is_empty() {
            return Err(Error::InvalidArgument(
                "Public key hash cannot be empty".to_string(),
            ));
        }
    }

    let username = new_username.unwrap_or(&current_user.username);
    let public_key = new_public_key.unwrap_or(&current_user.public_key);
    let public_key_hash = new_public_key_hash.unwrap_or(&current_user.public_key_hash);
    let updated_at = Utc::now().naive_utc();

    let mut args = SqliteArguments::default();

    args.add(username);
    args.add(public_key);
    args.add(public_key_hash);
    args.add(updated_at);
    args.add(user_id);

    sqlx::query_with(
        r#"
        UPDATE users 
        SET 
            username = $1,
            public_key = $2,
            public_key_hash = $3,
            updated_at = $4
        WHERE id = $5
        "#,
        args,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_anon_mapping(
    pool: &SqlitePool,
    anon_id: &str,
    user_id: Uuid,
    ttl_seconds: i64,
) -> Result<(), Error> {
    let now = Utc::now().timestamp();
    sqlx::query!(
        r#"
        INSERT INTO anon_mappings (anon_id, user_id, created_at, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        anon_id,
        user_id.to_string(),
        now,
        now + ttl_seconds,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn resolve_anon_id(pool: &SqlitePool, anon_id: &str) -> Result<Option<Uuid>, Error> {
    let now = Utc::now().timestamp();
    let record = sqlx::query!(
        r#"
        SELECT user_id FROM anon_mappings
        WHERE anon_id = $1 AND expires_at > $2
        "#,
        anon_id,
        now
    )
    .fetch_optional(pool)
    .await?;

    Ok(record.and_then(|r| Uuid::parse_str(&r.user_id).ok()))
}

pub async fn create_message(
    pool: &SqlitePool,
    sender_id: Uuid,
    recipient_id: Uuid,
    encrypted_content: &str,
    signature: Option<&str>,
    parent_id: Option<i64>,
) -> Result<Option<i64>, Error> {
    let conn = pool.acquire().await?;
    let current_time = Utc::now().timestamp();
    println!("{}", current_time);

    let message_id = sqlx::query!(
        r#"
        INSERT INTO messages (sender_id, recipient_id, encrypted_content, signature, parent_id, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
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
            sender_id as "sender_id: uuid::Uuid",
            recipient_id as "recipient_id: uuid::Uuid",
            encrypted_content, 
            signature, 
            parent_id, 
            is_read, 
            created_at
        FROM messages
        WHERE id = $1
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
        WHERE id = $1
        "#,
        message_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_conversation(
    pool: &SqlitePool,
    user1_id: Uuid,
    user2_id: Uuid,
    limit: Option<i64>,
) -> Result<Vec<Message>, Error> {
    #[derive(sqlx::FromRow)]
    struct DbMessage {
        id: i64,
        sender_id: Uuid,
        recipient_id: Uuid,
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
        WHERE (sender_id = $1 AND recipient_id = $2)
           OR (sender_id = $2 AND recipient_id = $1)
        ORDER BY created_at DESC
        LIMIT $3
        "#,
    )
    .bind(user1_id)
    .bind(user2_id)
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

pub async fn get_unread_messages(pool: &SqlitePool, user_id: Uuid) -> Result<Vec<Message>, Error> {
    #[derive(sqlx::FromRow)]
    struct DbMessage {
        id: i64,
        sender_id: Uuid,
        recipient_id: Uuid,
        encrypted_content: String,
        signature: Option<String>,
        parent_id: Option<i64>,
        is_read: i64,
        created_at: i64,
    }

    let unread_messages = sqlx::query_as::<_, DbMessage>(
        r#"
        SELECT 
            id, 
            sender_id,
            recipient_id,
            encrypted_content, 
            signature, 
            parent_id, 
            created_at,
            is_read
        FROM messages
        WHERE recipient_id = $1 AND is_read = 0
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
        SELECT
            id, 
            sender_id,
            recipient_id,
            encrypted_content, 
            signature, 
            parent_id, 
            created_at, 
            is_read
        FROM messages
        WHERE parent_id = $1
        ORDER BY created_at ASC
        LIMIT $2 OFFSET $3
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

/*
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
}*/

/// Gets a complete thread including the parent message and all nested replies (recursively),
/// ordered by timestamp ascending.
pub async fn get_complete_thread(
    pool: &SqlitePool,
    thread_root_id: i64,
    limit: Option<i64>,
) -> Result<Vec<Message>, Error> {
    let parent_message = match get_message(pool, thread_root_id).await? {
        Some(msg) => msg,
        None => return Err(Error::RowNotFound),
    };

    let mut messages = Vec::new();
    messages.push(parent_message.clone());

    // Recursive fetcher, boxed to allow async recursion
    fn fetch_replies_recursive<'a>(
        pool: &'a SqlitePool,
        parent_id: i64,
        limit: Option<i64>,
        acc: &'a mut Vec<Message>,
    ) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let replies = get_thread_replies(pool, parent_id, limit, None).await?;
            for reply in replies.iter() {
                acc.push(reply.clone());
                fetch_replies_recursive(pool, reply.id, limit, acc).await?;
            }
            Ok(())
        })
    }

    fetch_replies_recursive(pool, parent_message.id, limit, &mut messages).await?;

    messages.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    Ok(messages)
}

pub async fn get_user_threads(
    pool: &SqlitePool,
    user_id: Uuid,
    limit: Option<i64>,
) -> Result<Vec<Message>, Error> {
    let raw_messages = sqlx::query_as::<_, RawMessage>(
        r#"
        SELECT DISTINCT m.id, 
                m.sender_id, 
                m.recipient_id, 
                m.encrypted_content, 
                m.signature, m.parent_id, m.created_at, m.is_read
        FROM messages m
        JOIN messages r ON m.id = r.parent_id
        WHERE m.sender_id = $1 OR m.recipient_id = $1
        ORDER BY m.created_at DESC
        LIMIT $2
        "#,
    )
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
    user_id: Uuid,
    token_hash: &str,
    expires_at: i64,
    device_info: Option<&str>,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
            INSERT INTO refresh_tokens 
            (user_id, token_hash, expires_at, device_info)
            VALUES ($1, $2, datetime($3, 'unixepoch'), $4)
        "#,
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
    user_id: Uuid,
    token_hash: &str,
) -> Result<bool, Error> {
    let record = sqlx::query!(
        r#"
        SELECT id FROM refresh_tokens
        WHERE user_id = $1 AND token_hash = $2 AND expires_at > CURRENT_TIMESTAMP
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
        SELECT token_hash, $1 FROM refresh_tokens
        WHERE token_hash = $2
        "#,
        reason,
        token_hash
    )
    .execute(db)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM refresh_tokens
        WHERE token_hash = $1
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
