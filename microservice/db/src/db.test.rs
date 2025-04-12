use super::*;
use serial_test::serial;
use sqlx::migrate::MigrateDatabase;
use sqlx::Row;
use std::path::PathBuf;
use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

async fn setup_test_db() -> SqlitePool {
    let root_dir = PathBuf::from(env::current_dir().unwrap().parent().unwrap());
    let env_file_path = root_dir.join(".env.test");

    INIT.call_once(|| {
        dotenv::from_path(env_file_path).ok();
    });

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.test");

    if sqlx::Sqlite::database_exists(&database_url).await.unwrap_or(false) {
        sqlx::Sqlite::drop_database(&database_url).await.unwrap();
    }
    sqlx::Sqlite::create_database(&database_url).await.unwrap();

    let pool = SqlitePool::connect(&database_url).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT UNIQUE NOT NULL,
            public_key_hash TEXT NOT NULL,
            encrypted_private_key TEXT NOT NULL,
            encryption_salt TEXT NOT NULL,
            encryption_nonce TEXT NOT NULL
        );
        CREATE TABLE encrypted_messages (
            id SERIAL PRIMARY KEY,
            sender_id TEXT NOT NULL,
            recipient_id TEXT NOT NULL,
            encrypted_message TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            CONSTRAINT fk_sender
                FOREIGN KEY (sender_id)
                REFERENCES users(user_id)
                ON DELETE CASCADE,
            CONSTRAINT fk_recipient
                FOREIGN KEY (recipient_id)
                REFERENCES users(user_id)
                ON DELETE CASCADE
        );
        "
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

#[tokio::test]
#[serial]
async fn test_create_db_pool() {
    let pool = create_db_pool().await.unwrap();
    assert!(pool.acquire().await.is_ok());
}

#[tokio::test]
#[serial]
async fn test_insert_user() {
    let pool = setup_test_db().await;
    let user_id = "test_user";
    let public_key_hash = "test_hash";
    let encrypted_private_key = "encrypted_key";
    let encryption_salt = "salt";
    let encryption_nonce = "nonce";

    let result = insert_user(
        &pool,
        user_id,
        public_key_hash,
        encrypted_private_key,
        encryption_salt,
        encryption_nonce,
    )
    .await;
    assert!(result.is_ok());

    let user = get_user_by_id(&pool, user_id).await.unwrap();
    assert_eq!(user.user_id, user_id);
    assert_eq!(user.public_key_hash, public_key_hash);
    assert_eq!(user.encrypted_private_key, encrypted_private_key);
    assert_eq!(user.encryption_salt, encryption_salt);
    assert_eq!(user.encryption_nonce, encryption_nonce);
}

#[tokio::test]
#[serial]
async fn test_get_user_by_id() {
    let pool = setup_test_db().await;
    let user_id = "test_user_get";
    let public_key_hash = "test_hash_get";
    let encrypted_private_key = "encrypted_key_get";
    let encryption_salt = "salt_get";
    let encryption_nonce = "nonce_get";

    insert_user(
        &pool,
        user_id,
        public_key_hash,
        encrypted_private_key,
        encryption_salt,
        encryption_nonce,
    )
    .await
    .unwrap();

    let user = get_user_by_id(&pool, user_id).await.unwrap();
    assert_eq!(user.user_id, user_id);
    assert_eq!(user.public_key_hash, public_key_hash);
    assert_eq!(user.encrypted_private_key, encrypted_private_key);
    assert_eq!(user.encryption_salt, encryption_salt);
    assert_eq!(user.encryption_nonce, encryption_nonce);
}

#[tokio::test]
#[serial]
async fn test_get_user_by_id_not_found() {
    let pool = setup_test_db().await;
    let result = get_user_by_id(&pool, "nonexistent_user").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_generate_user_id() {
    let user_id = generate_user_id();
    assert_eq!(user_id.len(), 8);
    assert!(user_id.chars().all(|c| c.is_ascii_hexdigit()));
}

#[tokio::test]
async fn test_generate_user_id_uniqueness() {
    let mut ids = std::collections::HashSet::new();
    for _ in 0..1000 {
        let id = generate_user_id();
        assert!(ids.insert(id), "Generated duplicate ID");
    }
}

#[tokio::test]
#[serial]
async fn test_insert_user_with_retry_success() {
    let pool = setup_test_db().await;
    let user_id = "test_user_retry";
    let public_key_hash = "test_hash_retry";
    let encrypted_private_key = "encrypted_key_retry";
    let encryption_salt = "salt_retry";
    let encryption_nonce = "nonce_retry";

    let result = insert_user_with_retry(
        &pool,
        user_id,
        public_key_hash,
        encrypted_private_key,
        encryption_salt,
        encryption_nonce,
    )
    .await;
    assert!(result.is_ok());

    let user = get_user_by_id(&pool, user_id).await.unwrap();
    assert_eq!(user.user_id, user_id);
    assert_eq!(user.public_key_hash, public_key_hash);
    assert_eq!(user.encrypted_private_key, encrypted_private_key);
    assert_eq!(user.encryption_salt, encryption_salt);
    assert_eq!(user.encryption_nonce, encryption_nonce);
}

#[tokio::test]
#[serial]
async fn test_insert_user_with_retry_duplicate() {
    let pool = setup_test_db().await;
    let user_id = "test_user_dup";
    let public_key_hash = "test_hash_dup";
    let encrypted_private_key = "encrypted_key_dup";
    let encryption_salt = "salt_dup";
    let encryption_nonce = "nonce_dup";

    insert_user(
        &pool,
        user_id,
        public_key_hash,
        encrypted_private_key,
        encryption_salt,
        encryption_nonce,
    )
    .await
    .unwrap();

    let different_hash = "different_hash";
    let different_key = "different_key";
    let different_salt = "different_salt";
    let different_nonce = "different_nonce";

    let result = insert_user_with_retry(
        &pool,
        user_id,
        different_hash,
        different_key,
        different_salt,
        different_nonce,
    )
    .await;
    assert!(result.is_ok());
        
    let first_user = get_user_by_id(&pool, user_id).await.unwrap();
    assert_eq!(first_user.public_key_hash, public_key_hash);
    
    let query = "SELECT user_id FROM users WHERE public_key_hash = ?";
    let rows = sqlx::query(query)
        .bind(different_hash)
        .fetch_all(&pool)
        .await
        .unwrap();
    
    assert_eq!(rows.len(), 1);
    
    let new_user_id: String = rows[0].get(0);
    assert_ne!(new_user_id, user_id);
	println!("{} {}", new_user_id, user_id);
    
    let new_user = get_user_by_id(&pool, &new_user_id).await.unwrap();
    assert_eq!(new_user.public_key_hash, different_hash);
    assert_eq!(new_user.encrypted_private_key, different_key);
    assert_eq!(new_user.encryption_salt, different_salt);
    assert_eq!(new_user.encryption_nonce, different_nonce);
}

#[tokio::test]
#[serial]
async fn test_user_struct_serialization() {
    let user = User {
        user_id: "test_user".to_string(),
        public_key_hash: "test_hash".to_string(),
        encrypted_private_key: "encrypted_key".to_string(),
        encryption_salt: "salt".to_string(),
        encryption_nonce: "nonce".to_string(),
    };

    let serialized = serde_json::to_string(&user).unwrap();
    let deserialized: User = serde_json::from_str(&serialized).unwrap();

    assert_eq!(user.user_id, deserialized.user_id);
    assert_eq!(user.public_key_hash, deserialized.public_key_hash);
    assert_eq!(user.encrypted_private_key, deserialized.encrypted_private_key);
    assert_eq!(user.encryption_salt, deserialized.encryption_salt);
    assert_eq!(user.encryption_nonce, deserialized.encryption_nonce);
}

#[tokio::test]
#[serial]
async fn test_concurrent_user_insertion() {
    let pool = setup_test_db().await;
    let mut handles = vec![];

    for i in 0..10 {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            let user_id = format!("concurrent_user_{}", i);
            let public_key_hash = format!("concurrent_hash_{}", i);
            let encrypted_private_key = format!("concurrent_key_{}", i);
            let encryption_salt = format!("concurrent_salt_{}", i);
            let encryption_nonce = format!("concurrent_nonce_{}", i);

            insert_user_with_retry(
                &pool,
                &user_id,
                &public_key_hash,
                &encrypted_private_key,
                &encryption_salt,
                &encryption_nonce,
            )
            .await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}
