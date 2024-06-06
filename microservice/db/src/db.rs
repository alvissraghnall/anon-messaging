use uuid::Uuid;
use sqlx::{SqlitePool, Error};
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use std::env;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Option<i64>,
    pub user_id: String,
    pub public_key_hash: String,
}

pub async fn create_db_pool() -> Result<SqlitePool, Error> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqlitePool::connect(&database_url).await
}

pub async fn insert_user(pool: &SqlitePool, user_id: &str, public_key_hash: &str) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO users (user_id, public_key_hash) VALUES ($1, $2)",
        user_id,
        public_key_hash
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_user_by_id(pool: &SqlitePool, user_id: &str) -> Result<User, Error> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, user_id, public_key_hash FROM users WHERE user_id = $1",
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
) -> Result<(), String> {
    let mut retries = 0;
    let mut final_user_id = user_id.to_string();

    loop {
	    match insert_user(pool, &final_user_id, public_key_hash).await {
             Ok(_) => return Ok(()),
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


#[cfg(test)]
#[path = "db.test.rs"]
mod tests;
