use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::DateTime;
use sqlx::{Error, SqlitePool};
use std::env;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: i64,
    pub sender_id: String,
    pub recipient_id: String,
    pub encrypted_content: String,
    pub parent_id: Option<i64>,
    pub signature: Option<String>,
    pub created_at: DateTime,
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
    created_at: i64,
) -> Result<i64, Error> {
    let mut conn = pool.acquire().await?;

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
        created_at
    )
    .fetch_one(&mut conn)
    .await?
    .id;

    Ok(message_id)
}

pub async fn get_message(pool: &SqlitePool, message_id: i64) -> Result<Option<Message>, Error> {
    sqlx::query_as!(
        Message,
        r#"
        SELECT id, sender_id, recipient_id, encrypted_content, signature, parent_id, created_at, is_read
        FROM messages
        WHERE id = ?
        "#,
        message_id
    )
    .fetch_optional(pool)
    .await
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
    let messages = sqlx::query_as!(
        Message,
        r#"
        SELECT id, sender_id, recipient_id, encrypted_content, signature, parent_id, created_at, is_read
        FROM messages
        WHERE (sender_id = ? AND recipient_id = ?)
           OR (sender_id = ? AND recipient_id = ?)
        ORDER BY created_at DESC
        LIMIT ?
        "#,
        user1_id,
        user2_id,
        user2_id,
        user1_id,
        limit.unwrap_or(100)
    )
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

pub async fn get_unread_messages(pool: &SqlitePool, user_id: &str) -> Result<Vec<Message>, Error> {
    sqlx::query_as!(
        Message,
        r#"
        SELECT id, sender_id, recipient_id, encrypted_content, signature, parent_id, created_at,
        CASE 
            WHEN is_read = 1 THEN TRUE 
            ELSE FALSE
        END AS is_read
        FROM messages
        WHERE recipient_id = ? AND is_read = 0
        ORDER BY created_at ASC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
}

#[cfg(test)]
#[path = "db.test.rs"]
mod tests;
