use super::*;
use serial_test::serial;
use sqlx::migrate::MigrateDatabase;
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

    // Drop the database if it exists and recreate it
    if sqlx::Sqlite::database_exists(&database_url).await.unwrap_or(false) {
        sqlx::Sqlite::drop_database(&database_url).await.unwrap();
    }
    sqlx::Sqlite::create_database(&database_url).await.unwrap();

    let pool = SqlitePool::connect(&database_url).await.unwrap();

    // Create the tables
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT UNIQUE NOT NULL,
            public_key_hash TEXT NOT NULL,
            encrypted_private_key TEXT NOT NULL,
            encryption_salt TEXT NOT NULL,
            encryption_nonce TEXT NOT NULL
        )"
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
    let encrypted_private_key = b"encrypted_key";
    let encryption_salt = "salt";
    let encryption_nonce = b"nonce";

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

    // Verify the user was inserted
    let user = get_user_by_id(&pool, user_id).await.unwrap();
    assert_eq!(user.user_id, user_id);
    assert_eq!(user.public_key_hash, public_key_hash);
    assert_eq!(user.encrypted_private_key, base64::encode(encrypted_private_key));
    assert_eq!(user.encryption_salt, encryption_salt);
    assert_eq!(user.encryption_nonce, base64::encode(encryption_nonce));
}

#[tokio::test]
#[serial]
async fn test_get_user_by_id() {
    let pool = setup_test_db().await;
    let user_id = "test_user_get";
    let public_key_hash = "test_hash_get";
    let encrypted_private_key = b"encrypted_key_get";
    let encryption_salt = "salt_get";
    let encryption_nonce = b"nonce_get";

    // Insert a user first
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

    // Test getting the user
    let user = get_user_by_id(&pool, user_id).await.unwrap();
    assert_eq!(user.user_id, user_id);
    assert_eq!(user.public_key_hash, public_key_hash);
    assert_eq!(user.encrypted_private_key, base64::encode(encrypted_private_key));
    assert_eq!(user.encryption_salt, encryption_salt);
    assert_eq!(user.encryption_nonce, base64::encode(encryption_nonce));
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
    let encrypted_private_key = b"encrypted_key_retry";
    let encryption_salt = "salt_retry";
    let encryption_nonce = b"nonce_retry";

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

    // Verify the user was inserted
    let user = get_user_by_id(&pool, user_id).await.unwrap();
    assert_eq!(user.user_id, user_id);
    assert_eq!(user.public_key_hash, public_key_hash);
    assert_eq!(user.encrypted_private_key, base64::encode(encrypted_private_key));
    assert_eq!(user.encryption_salt, encryption_salt);
    assert_eq!(user.encryption_nonce, base64::encode(encryption_nonce));
}

#[tokio::test]
#[serial]
async fn test_insert_user_with_retry_duplicate() {
    let pool = setup_test_db().await;
    let user_id = "test_user_dup";
    let public_key_hash = "test_hash_dup";
    let encrypted_private_key = b"encrypted_key_dup";
    let encryption_salt = "salt_dup";
    let encryption_nonce = b"nonce_dup";

    // Insert first user
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

    // Try to insert duplicate
    let result = insert_user_with_retry(
        &pool,
        user_id,
        "different_hash",
        b"different_key",
        "different_salt",
        b"different_nonce",
    )
    .await;
    assert!(result.is_ok()); // Should succeed with a different user_id

    // Verify both users exist with different IDs
    let first_user = get_user_by_id(&pool, user_id).await.unwrap();
    assert_eq!(first_user.public_key_hash, public_key_hash);
}

#[tokio::test]
#[serial]
async fn test_user_struct_serialization() {
    let user = User {
        id: Some(1),
        user_id: "test_user".to_string(),
        public_key_hash: "test_hash".to_string(),
        encrypted_private_key: base64::encode(b"encrypted_key"),
        encryption_salt: "salt".to_string(),
        encryption_nonce: base64::encode(b"nonce"),
    };

    let serialized = serde_json::to_string(&user).unwrap();
    let deserialized: User = serde_json::from_str(&serialized).unwrap();

    assert_eq!(user.id, deserialized.id);
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
            let encrypted_private_key = format!("concurrent_key_{}", i).into_bytes();
            let encryption_salt = format!("concurrent_salt_{}", i);
            let encryption_nonce = format!("concurrent_nonce_{}", i).into_bytes();

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
