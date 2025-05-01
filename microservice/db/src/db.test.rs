use super::*;
use faker_rand::en_us::names::FullName;
use p256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use rand::Rng;
use serial_test::serial;
use sha2::{Digest, Sha256};
use sqlx::migrate::MigrateDatabase;
use sqlx::Row;
use std::sync::Once;
use tokio::time::Duration;

static INIT: Once = Once::new();

async fn setup_test_db() -> SqlitePool {
    let root_dir = PathBuf::from(env::current_dir().unwrap().parent().unwrap());
    let env_file_path = root_dir.join(".env.test");
    INIT.call_once(|| {
        dotenv::from_path(env_file_path).expect("ENV TEST FILE MUST EXIST!");
    });

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.test");
    println!("{database_url}");

    if sqlx::Sqlite::database_exists(&database_url)
        .await
        .unwrap_or(false)
    {
        sqlx::Sqlite::drop_database(&database_url).await.unwrap();
    }
    sqlx::Sqlite::create_database(&database_url).await.unwrap();

    let pool = SqlitePool::connect(&database_url).await.unwrap();

    sqlx::query(
        "
        DROP TABLE IF EXISTS users;

        CREATE TABLE users (
            id TEXT PRIMARY KEY NOT NULL,
            username TEXT NOT NULL UNIQUE,
            public_key TEXT NOT NULL UNIQUE,
            public_key_hash TEXT NOT NULL UNIQUE,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_login TIMESTAMP
        );
      
	DROP TABLE IF EXISTS messages;

        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sender_id TEXT NOT NULL,
            recipient_id TEXT NOT NULL,
            encrypted_content TEXT NOT NULL,
            parent_id INTEGER,
            signature TEXT,
            is_read INTEGER DEFAULT 0 NOT NULL,
            created_at INTEGER NOT NULL,
            CONSTRAINT fk_sender
                FOREIGN KEY (sender_id)
                REFERENCES users(id)
                ON DELETE CASCADE,
            CONSTRAINT fk_recipient
                FOREIGN KEY (recipient_id)
                REFERENCES users(id)
                ON DELETE CASCADE
        );

	DROP TABLE IF EXISTS revoked_tokens;

        CREATE TABLE IF NOT EXISTS revoked_tokens (
            token_hash TEXT PRIMARY KEY NOT NULL,
            revoked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            reason TEXT
        );

	DROP TABLE IF EXISTS refresh_token;

        CREATE TABLE IF NOT EXISTS refresh_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            token_hash TEXT NOT NULL UNIQUE,
            device_info TEXT,
            expires_at TIMESTAMP NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        ",
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

#[tokio::test]
#[serial]
async fn test_insert_user() {
    let pool = setup_test_db().await;
    let public_key_hash = "test_hash";
    let public_key = "encrypted_key";
    let username = "salt";

    let result = insert_user(&pool, public_key_hash, public_key, username).await;
    println!("{:?}", result);
    assert!(result.is_ok());

    let user = get_user_by_id(&pool, result.unwrap()).await.unwrap();
    assert_eq!(user.id.get_version(), Some(uuid::Version::SortRand));
    assert_eq!(user.public_key_hash, public_key_hash);
    assert_eq!(user.public_key, public_key);
    assert_eq!(user.username, username);
    assert_eq!(user.last_login, None);

    let now = Utc::now().naive_utc();
    assert!(user.created_at <= now);
    assert!(user.created_at >= now - chrono::Duration::seconds(5));
    assert!(user.updated_at <= now);
    assert!(user.updated_at >= now - chrono::Duration::seconds(5));
}

#[tokio::test]
#[serial]
async fn test_get_user_by_id() {
    let pool = setup_test_db().await;
    let username = "test_user_get";
    let public_key_hash = "test_hash_get";
    let public_key = "encrypted_key_get";

    let user_id = insert_user(&pool, public_key_hash, public_key, username)
        .await
        .unwrap();
    let user = get_user_by_id(&pool, user_id).await.unwrap();
    assert_eq!(user.id.get_version(), Some(uuid::Version::SortRand));
    assert_eq!(user.public_key_hash, public_key_hash);
    assert_eq!(user.public_key, public_key);
}

#[tokio::test]
#[serial]
async fn test_get_user_by_id_not_found() {
    let pool = setup_test_db().await;
    let result = get_user_by_id(&pool, Uuid::now_v7()).await;
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_user_struct_serialization() {
    let user = User {
        id: Uuid::now_v7(),
        username: "test_user".to_string(),
        public_key_hash: "test_hash".to_string(),
        public_key: "encrypted_key".to_string(),
        created_at: Utc::now().naive_utc(),
        last_login: Option::Some(Utc::now().naive_utc()),
        updated_at: Utc::now().naive_utc(),
    };

    let serialized = serde_json::to_string(&user).unwrap();
    let deserialized: User = serde_json::from_str(&serialized).unwrap();

    assert_eq!(user.username, deserialized.username);
    assert_eq!(user.public_key_hash, deserialized.public_key_hash);
    assert_eq!(user.public_key, deserialized.public_key);

    assert!((user.created_at - deserialized.created_at).num_seconds() <= 1);
    assert!((user.updated_at - deserialized.updated_at).num_seconds() <= 1);
    assert_eq!(user.id, deserialized.id);
    assert!(user.last_login.is_some() && deserialized.last_login.is_some());
}

#[tokio::test]
#[serial]
async fn test_concurrent_user_insertion() {
    let pool = setup_test_db().await;
    let mut handles = vec![];

    for i in 0..10 {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            let username = format!("concurrent_user_{}", i);
            let public_key_hash = format!("concurrent_hash_{}", i);
            let public_key = format!("concurrent_key_{}", i);

            insert_user(&pool, &public_key_hash, &public_key, &username).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_get_user_by_pubkey_found() {
    let pool = setup_test_db().await;

    let username = "test_user";
    let public_key = "test_public_key";
    let public_key_hash = "test_public_key_hash";

    insert_user(&pool, public_key_hash, public_key, username)
        .await
        .unwrap();

    let result = get_user_by_pubkey(&pool, public_key_hash).await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.username, username);
    assert_eq!(user.public_key, public_key);
    assert_eq!(user.public_key_hash, public_key_hash);
}

#[tokio::test]
async fn test_get_user_by_pubkey_not_found() {
    let pool = setup_test_db().await;

    let result = get_user_by_pubkey(&pool, "nonexistent_hash").await;

    assert!(result.is_err());

    assert!(matches!(result.unwrap_err(), Error::RowNotFound));
}

#[tokio::test]
async fn test_get_user_by_pubkey_special_characters() {
    let pool = setup_test_db().await;

    let user_id = Uuid::now_v7();
    let username = "special_chars_user";
    let public_key = "special_public_key";
    let public_key_hash = "Hash!@#$%^&*()_+-=[]{}|;':,./<>?";

    insert_user(&pool, public_key_hash, public_key, username)
        .await
        .unwrap();

    let result = get_user_by_pubkey(&pool, public_key_hash).await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.username, username);
    assert_eq!(user.public_key_hash, public_key_hash);
    assert_eq!(user.public_key, public_key);
}

#[tokio::test]
async fn test_fetch_public_key_hash_success() {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();

    create_test_user(&pool, user_id).await.unwrap();

    let original_user = get_user_by_id(&pool, user_id).await.unwrap();

    let result = fetch_public_key_hash(&pool, user_id).await;

    assert!(result.is_ok());

    let public_key_hash = result.unwrap();
    assert_eq!(public_key_hash, original_user.public_key_hash);
}

#[tokio::test]
async fn test_fetch_public_key_hash_user_not_found() {
    let pool = setup_test_db().await;

    let nonexistent_user_id = Uuid::now_v7();

    let result = fetch_public_key_hash(&pool, nonexistent_user_id).await;

    assert!(result.is_err());

    match result {
        Err(sqlx::Error::RowNotFound) => {}
        _ => {
            panic!("Expected RowNotFound error, got: {:?}", result);
        }
    }
}

#[tokio::test]
async fn test_create_message() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let sender_id = Uuid::now_v7();
    let recipient_id = Uuid::now_v7();

    create_test_user(&pool, sender_id).await?;
    create_test_user(&pool, recipient_id).await?;

    let encrypted_content = "encrypted test message content";
    let signature = Some("test signature");
    let parent_id = None;

    let message_id = create_message(
        &pool,
        sender_id,
        recipient_id,
        encrypted_content,
        signature,
        parent_id,
    )
    .await?;

    assert!(message_id.is_some());
    assert!(message_id.unwrap() > 0);

    Ok(())
}

#[tokio::test]
async fn test_get_message() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let sender_id = Uuid::now_v7();
    let recipient_id = Uuid::now_v7();
    create_test_user(&pool, sender_id).await;
    create_test_user(&pool, recipient_id).await;
    let encrypted_content = "test message content";
    let signature = Some("test signature");
    let parent_id = None;

    let message_id = create_message(
        &pool,
        sender_id,
        recipient_id,
        encrypted_content,
        signature,
        parent_id,
    )
    .await?
    .unwrap();

    let message = get_message(&pool, message_id).await?;

    assert!(message.is_some());
    let message = message.unwrap();
    assert_eq!(message.id, message_id);
    assert_eq!(message.sender_id, sender_id);
    assert_eq!(message.recipient_id, recipient_id);
    assert_eq!(message.encrypted_content, encrypted_content);
    assert_eq!(message.signature, Some(signature.unwrap().to_string()));
    assert_eq!(message.parent_id, parent_id);
    assert!(!message.is_read);

    Ok(())
}

#[tokio::test]
async fn test_get_nonexistent_message() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let message = get_message(&pool, 999999).await?;

    // Assert that no message was found
    assert!(message.is_none());

    Ok(())
}

#[tokio::test]
async fn test_create_message_with_parent() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let sender_id = Uuid::now_v7();
    let recipient_id = Uuid::now_v7();
    create_test_user(&pool, sender_id).await;
    create_test_user(&pool, recipient_id).await;
    let encrypted_content = "parent message";

    // Create parent message
    let parent_id = create_message(
        &pool,
        sender_id,
        recipient_id,
        encrypted_content,
        None,
        None,
    )
    .await?
    .unwrap();

    // Create child message referencing the parent
    let child_message_id = create_message(
        &pool,
        recipient_id,
        sender_id,
        "child message",
        None,
        Some(parent_id),
    )
    .await?
    .unwrap();

    // Retrieve the child message
    let child_message = get_message(&pool, child_message_id).await?.unwrap();

    assert_eq!(child_message.parent_id, Some(parent_id));

    Ok(())
}

#[tokio::test]
async fn test_create_message_with_invalid_parent() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let sender_id = Uuid::now_v7();
    let recipient_id = Uuid::now_v7();
    let encrypted_content = "test message with invalid parent";

    let result = create_message(
        &pool,
        sender_id,
        recipient_id,
        encrypted_content,
        None,
        Some(9999),
    )
    .await;

    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_mark_message_read() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let sender_id = Uuid::now_v7();
    let recipient_id = Uuid::now_v7();

    create_test_user(&pool, sender_id).await;
    create_test_user(&pool, recipient_id).await;
    let message_id = create_message(
        &pool,
        sender_id,
        recipient_id,
        "test message content",
        None,
        None,
    )
    .await?
    .unwrap();

    let message_before = get_message(&pool, message_id).await?.unwrap();
    assert!(!message_before.is_read);

    mark_message_read(&pool, message_id).await?;

    let message_after = get_message(&pool, message_id).await?.unwrap();
    assert!(message_after.is_read);

    Ok(())
}

#[tokio::test]
async fn test_mark_nonexistent_message_read() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let result = mark_message_read(&pool, 99999).await;

    // This should not return an error, as SQLite UPDATE statements don't error
    // when no rows are affected
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_get_conversation_empty() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let user1_id = Uuid::now_v7();
    let user2_id = Uuid::now_v7();

    create_test_user(&pool, user1_id).await;
    create_test_user(&pool, user2_id).await;
    let messages = get_conversation(&pool, user1_id, user2_id, None).await?;

    assert!(messages.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_conversation_with_messages() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let user1_id = Uuid::now_v7();
    let user2_id = Uuid::now_v7();
    let user3_id = Uuid::now_v7();

    create_test_user(&pool, user1_id).await;
    create_test_user(&pool, user2_id).await;
    create_test_user(&pool, user3_id).await;

    let msg1_id = create_message(
        &pool,
        user1_id,
        user2_id,
        "Message 1 from user1 to user2",
        None,
        None,
    )
    .await?
    .unwrap();

    let msg2_id = create_message(
        &pool,
        user2_id,
        user1_id,
        "Message 2 from user2 to user1",
        None,
        None,
    )
    .await?
    .unwrap();

    create_message(
        &pool,
        user1_id,
        user3_id,
        "Message not in conversation",
        None,
        None,
    )
    .await?
    .unwrap();

    let messages = get_conversation(&pool, user1_id, user2_id, None).await?;

    assert_eq!(messages.len(), 2);

    let message_ids: Vec<i64> = messages.iter().map(|m| m.id).collect();
    assert!(message_ids.contains(&msg1_id));
    assert!(message_ids.contains(&msg2_id));

    for msg in messages {
        assert!(
            (msg.sender_id == user1_id && msg.recipient_id == user2_id)
                || (msg.sender_id == user2_id && msg.recipient_id == user1_id)
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_get_conversation_with_limit() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let user1_id = Uuid::now_v7();
    let user2_id = Uuid::now_v7();
    create_test_user(&pool, user1_id).await;
    create_test_user(&pool, user2_id).await;

    for i in 0..5 {
        create_message(
            &pool,
            user1_id,
            user2_id,
            &format!("Message {}", i),
            None,
            None,
        )
        .await?
        .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let messages = get_conversation(&pool, user1_id, user2_id, Some(3)).await?;

    assert_eq!(messages.len(), 3);

    // Verify messages are in descending order by created_at
    let mut prev_timestamp = Utc::now();
    for msg in &messages {
        assert!(msg.created_at <= prev_timestamp);
        prev_timestamp = msg.created_at;
    }

    Ok(())
}

#[tokio::test]
async fn test_get_conversation_bidirectional() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let user1_id = Uuid::now_v7();
    let user2_id = Uuid::now_v7();
    create_test_user(&pool, user1_id).await;
    create_test_user(&pool, user2_id).await;

    create_message(&pool, user1_id, user2_id, "From user1 to user2", None, None)
        .await?
        .unwrap();

    create_message(&pool, user2_id, user1_id, "From user2 to user1", None, None)
        .await?
        .unwrap();

    let messages1 = get_conversation(&pool, user1_id, user2_id, None).await?;

    let messages2 = get_conversation(&pool, user2_id, user1_id, None).await?;

    assert_eq!(messages1.len(), 2);
    assert_eq!(messages2.len(), 2);

    let ids1: Vec<i64> = messages1.iter().map(|m| m.id).collect();
    let ids2: Vec<i64> = messages2.iter().map(|m| m.id).collect();
    assert_eq!(ids1.len(), ids2.len());

    for id in ids1 {
        assert!(ids2.contains(&id));
    }

    Ok(())
}

#[tokio::test]
async fn test_get_unread_messages_empty() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let user_id = Uuid::now_v7();
    create_test_user(&pool, user_id).await;

    let messages = get_unread_messages(&pool, user_id).await?;

    assert!(messages.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_unread_messages_mixed() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let recipient_id = Uuid::now_v7();
    let sender1_id = Uuid::now_v7();
    let sender2_id = Uuid::now_v7();
    create_test_user(&pool, recipient_id).await;
    create_test_user(&pool, sender1_id).await;
    create_test_user(&pool, sender2_id).await;

    let msg1_id = create_message(
        &pool,
        sender1_id,
        recipient_id,
        "Unread message 1",
        None,
        None,
    )
    .await?
    .unwrap();

    let msg2_id = create_message(
        &pool,
        sender2_id,
        recipient_id,
        "Unread message 2",
        None,
        None,
    )
    .await?
    .unwrap();

    let read_msg_id = create_message(&pool, sender1_id, recipient_id, "Read message", None, None)
        .await?
        .unwrap();

    mark_message_read(&pool, read_msg_id).await?;

    create_message(
        &pool,
        recipient_id,
        sender1_id,
        "Message from recipient",
        None,
        None,
    )
    .await?;

    let unread_messages = get_unread_messages(&pool, recipient_id).await?;

    assert_eq!(unread_messages.len(), 2);

    // Verify the returned messages are the expected unread ones
    let message_ids: Vec<i64> = unread_messages.iter().map(|m| m.id).collect();
    assert!(message_ids.contains(&msg1_id));
    assert!(message_ids.contains(&msg2_id));
    assert!(!message_ids.contains(&read_msg_id));

    // Verify all messages are unread and to the recipient
    for msg in unread_messages {
        assert!(!msg.is_read);
        assert_eq!(msg.recipient_id, recipient_id);
    }

    Ok(())
}

#[tokio::test]
async fn test_get_unread_messages_order() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let recipient_id = Uuid::now_v7();
    let sender_id = Uuid::now_v7();
    create_test_user(&pool, recipient_id).await;
    create_test_user(&pool, sender_id).await;
    for i in 0..3 {
        create_message(
            &pool,
            sender_id,
            recipient_id,
            &format!("Message {}", i),
            None,
            None,
        )
        .await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let messages = get_unread_messages(&pool, recipient_id).await?;

    // Verify messages are in ascending order by created_at
    assert_eq!(messages.len(), 3);
    for i in 1..messages.len() {
        assert!(messages[i - 1].created_at <= messages[i].created_at);
    }

    Ok(())
}

#[tokio::test]
async fn test_get_thread_replies_empty() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let user1_id = Uuid::now_v7();
    let user2_id = Uuid::now_v7();
    create_test_user(&pool, user1_id).await;
    create_test_user(&pool, user2_id).await;

    let parent_id = create_message(&pool, user1_id, user2_id, "Parent message", None, None)
        .await?
        .unwrap();

    let replies = get_thread_replies(&pool, parent_id, None, None).await?;

    assert!(replies.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_thread_replies_with_replies() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let user1_id = Uuid::now_v7();
    let user2_id = Uuid::now_v7();
    create_test_user(&pool, user1_id).await;
    create_test_user(&pool, user2_id).await;

    let parent_id = create_message(&pool, user1_id, user2_id, "Parent message", None, None)
        .await?
        .unwrap();

    let reply1_id = create_message(&pool, user2_id, user1_id, "Reply 1", None, Some(parent_id))
        .await?
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let reply2_id = create_message(&pool, user1_id, user2_id, "Reply 2", None, Some(parent_id))
        .await?
        .unwrap();

    let other_parent_id = create_message(&pool, user1_id, user2_id, "Another parent", None, None)
        .await?
        .unwrap();

    let other_reply_id = create_message(
        &pool,
        user2_id,
        user1_id,
        "Reply to different parent",
        None,
        Some(other_parent_id),
    )
    .await?
    .unwrap();

    // Get replies to the first thread
    let replies = get_thread_replies(&pool, parent_id, None, None).await?;

    assert_eq!(replies.len(), 2);

    let reply_ids: Vec<i64> = replies.iter().map(|m| m.id).collect();
    assert!(reply_ids.contains(&reply1_id));
    assert!(reply_ids.contains(&reply2_id));
    assert!(!reply_ids.contains(&other_reply_id));

    for reply in &replies {
        assert_eq!(reply.parent_id, Some(parent_id));
    }

    // Verify messages are in ascending order by created_at
    assert_eq!(replies[0].id, reply1_id);
    assert_eq!(replies[1].id, reply2_id);

    Ok(())
}

#[tokio::test]
async fn test_get_thread_replies_with_limit_offset() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let user1_id = Uuid::now_v7();
    let user2_id = Uuid::now_v7();
    create_test_user(&pool, user1_id).await;
    create_test_user(&pool, user2_id).await;

    let parent_id = create_message(&pool, user1_id, user2_id, "Parent message", None, None)
        .await?
        .unwrap();

    let mut reply_ids = Vec::with_capacity(5);
    for i in 0..5 {
        let sender = if i % 2 == 0 { user1_id } else { user2_id };
        let recipient = if i % 2 == 0 { user2_id } else { user1_id };

        let reply_id = create_message(
            &pool,
            sender,
            recipient,
            &format!("Reply {}", i),
            None,
            Some(parent_id),
        )
        .await?
        .unwrap();

        reply_ids.push(reply_id);

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // limit of 2
    let limited_replies = get_thread_replies(&pool, parent_id, Some(2), None).await?;
    assert_eq!(limited_replies.len(), 2);
    assert_eq!(limited_replies[0].id, reply_ids[0]);
    assert_eq!(limited_replies[1].id, reply_ids[1]);

    // offset of 2
    let offset_replies = get_thread_replies(&pool, parent_id, None, Some(2)).await?;
    assert_eq!(offset_replies.len(), 3); // the remaining 3 messages
    assert_eq!(offset_replies[0].id, reply_ids[2]);

    // both limit and offset
    let limited_offset_replies = get_thread_replies(&pool, parent_id, Some(2), Some(1)).await?;
    assert_eq!(limited_offset_replies.len(), 2);
    assert_eq!(limited_offset_replies[0].id, reply_ids[1]);
    assert_eq!(limited_offset_replies[1].id, reply_ids[2]);

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_store_and_validate_refresh_token() -> Result<(), Error> {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();
    create_test_user(&pool, user_id).await;
    let token_hash = "test_token_hash";
    let expires_at = Utc::now().timestamp() + 3600;

    store_refresh_token(&pool, user_id, token_hash, expires_at, Some("Test Device")).await?;

    let is_valid = validate_refresh_token(&pool, user_id, token_hash).await?;
    assert!(is_valid);

    let is_valid = validate_refresh_token(&pool, user_id, "wrong_hash").await?;
    assert!(!is_valid);

    let is_valid = validate_refresh_token(&pool, Uuid::now_v7(), token_hash).await?;
    assert!(!is_valid);

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_expired_token_validation() -> Result<(), Error> {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();
    create_test_user(&pool, user_id).await;
    let token_hash = "expired_token_hash";
    let expires_at = Utc::now().timestamp() - 3600;

    store_refresh_token(&pool, user_id, token_hash, expires_at, None).await?;

    let is_valid = validate_refresh_token(&pool, user_id, token_hash).await?;
    assert!(!is_valid, "Expired token should not validate");

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_revoke_refresh_token() -> Result<(), Error> {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();
    create_test_user(&pool, user_id).await;
    let token_hash = "token_to_revoke";
    let expires_at = Utc::now().timestamp() + 3600;

    store_refresh_token(&pool, user_id, token_hash, expires_at, None).await?;

    let is_valid = validate_refresh_token(&pool, user_id, token_hash).await?;
    assert!(is_valid);

    revoke_refresh_token(&pool, token_hash, Some("test revocation")).await?;

    // Verify token no longer validates
    let is_valid = validate_refresh_token(&pool, user_id, token_hash).await?;
    assert!(!is_valid);

    // Check revoked tokens table
    let revoked = sqlx::query!(
        "SELECT token_hash FROM revoked_tokens WHERE token_hash = ?",
        token_hash
    )
    .fetch_optional(&pool)
    .await?;
    assert!(revoked.is_some(), "Token should be in revoked table");

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_cleanup_expired_tokens() -> Result<(), Error> {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();
    create_test_user(&pool, user_id).await;

    store_refresh_token(
        &pool,
        user_id,
        "valid_hash",
        Utc::now().timestamp() + 3600,
        None,
    )
    .await?;

    store_refresh_token(
        &pool,
        user_id,
        "expired_hash",
        Utc::now().timestamp() - 3600,
        None,
    )
    .await?;

    let cleaned = cleanup_expired_tokens(&pool).await?;
    assert_eq!(cleaned, 1, "Should clean up 1 expired token");

    let remaining = sqlx::query!(
        "SELECT COUNT(*) as count FROM refresh_tokens WHERE user_id = ?",
        user_id
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(remaining.count, 1, "Only valid token should remain");

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_token_uniqueness() -> Result<(), Error> {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();
    create_test_user(&pool, user_id).await;

    let token_hash = "unique_hash";
    let expires_at = Utc::now().timestamp() + 3600;

    store_refresh_token(&pool, user_id, token_hash, expires_at, None).await?;

    let result = store_refresh_token(&pool, user_id, token_hash, expires_at, None).await;

    assert!(result.is_err(), "Should not allow duplicate token hashes");

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_revoked_token_cannot_validate() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let user_id = Uuid::now_v7();
    create_test_user(&pool, user_id).await?;

    let token_hash = "revocable_token_hash";
    let expires_at = Utc::now().timestamp() + 3600;

    store_refresh_token(&pool, user_id, token_hash, expires_at, Some("Test Device")).await?;

    let is_valid = validate_refresh_token(&pool, user_id, token_hash).await?;
    assert!(is_valid, "Token should validate before revocation");

    revoke_refresh_token(&pool, token_hash, Some("test revocation")).await?;

    let is_valid = validate_refresh_token(&pool, user_id, token_hash).await?;
    assert!(!is_valid, "Revoked token should not validate");

    // Verify token appears in revoked table
    let revoked = sqlx::query!(
        "SELECT reason FROM revoked_tokens WHERE token_hash = ?",
        token_hash
    )
    .fetch_optional(&pool)
    .await?;

    assert_eq!(revoked.unwrap().reason, Some("test revocation".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_update_user_no_changes() {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();

    create_test_user(&pool, user_id).await.unwrap();

    let original_user = get_user_by_id(&pool, user_id).await.unwrap();

    let result = update_user(&pool, user_id, None, None, None).await;

    assert!(result.is_ok());

    let updated_user = get_user_by_id(&pool, user_id).await.unwrap();

    assert_eq!(updated_user.username, original_user.username);
    assert_eq!(updated_user.public_key, original_user.public_key);
    assert_eq!(updated_user.public_key_hash, original_user.public_key_hash);

    assert!(updated_user.updated_at > original_user.updated_at);
}

#[tokio::test]
async fn test_update_user_username() {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();

    create_test_user(&pool, user_id).await.unwrap();

    let original_user = get_user_by_id(&pool, user_id).await.unwrap();

    let new_username = "updated_username";
    let result = update_user(&pool, user_id, Some(new_username), None, None).await;

    assert!(result.is_ok());

    let updated_user = get_user_by_id(&pool, user_id).await.unwrap();

    assert_eq!(updated_user.username, new_username);

    assert_eq!(updated_user.public_key, original_user.public_key);
    assert_eq!(updated_user.public_key_hash, original_user.public_key_hash);

    assert!(updated_user.updated_at > original_user.updated_at);
}

#[tokio::test]
async fn test_update_user_public_key() {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();

    create_test_user(&pool, user_id).await.unwrap();

    let original_user = get_user_by_id(&pool, user_id).await.unwrap();

    let new_public_key = "updated_public_key";
    let new_public_key_hash = "updated_public_key_hash";
    let result = update_user(
        &pool,
        user_id,
        None,
        Some(new_public_key),
        Some(new_public_key_hash),
    )
    .await;

    assert!(result.is_ok());

    let updated_user = get_user_by_id(&pool, user_id).await.unwrap();

    assert_eq!(updated_user.public_key, new_public_key);
    assert_eq!(updated_user.public_key_hash, new_public_key_hash);

    assert_eq!(updated_user.username, original_user.username);

    assert!(updated_user.updated_at > original_user.updated_at);
}

#[tokio::test]
async fn test_update_user_all_fields() {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();

    create_test_user(&pool, user_id).await.unwrap();

    let new_username = "completely_updated_user";
    let new_public_key = "completely_updated_public_key";
    let new_public_key_hash = "completely_updated_public_key_hash";

    let result = update_user(
        &pool,
        user_id,
        Some(new_username),
        Some(new_public_key),
        Some(new_public_key_hash),
    )
    .await;

    assert!(result.is_ok());

    let updated_user = get_user_by_id(&pool, user_id).await.unwrap();

    assert_eq!(updated_user.username, new_username);
    assert_eq!(updated_user.public_key, new_public_key);
    assert_eq!(updated_user.public_key_hash, new_public_key_hash);
}

#[tokio::test]
async fn test_update_nonexistent_user() {
    let pool = setup_test_db().await;
    let nonexistent_user_id = Uuid::now_v7();

    let result = update_user(&pool, nonexistent_user_id, Some("new_name"), None, None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_user_with_empty_values() {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();

    create_test_user(&pool, user_id).await.unwrap();

    let original_user = get_user_by_id(&pool, user_id).await.unwrap();

    let empty_username = "";
    let empty_public_key = "";
    let empty_public_key_hash = "";

    let result = update_user(
        &pool,
        user_id,
        Some(empty_username),
        Some(empty_public_key),
        Some(empty_public_key_hash),
    )
    .await;

    assert!(result.is_err());

    let updated_user = get_user_by_id(&pool, user_id).await.unwrap();

    assert_eq!(updated_user.username, original_user.username);
    assert_eq!(updated_user.public_key, original_user.public_key);
    assert_eq!(updated_user.public_key_hash, original_user.public_key_hash);
}

#[tokio::test]
async fn test_user_with_no_messages_returns_empty() -> Result<(), Box<dyn std::error::Error>> {
    let pool = setup_test_db().await;
    let user_id = Uuid::now_v7();
    create_test_user(&pool, user_id).await?;

    let threads = get_user_threads(&pool, user_id, Some(10)).await?;
    assert!(threads.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_user_with_single_thread() -> Result<(), Box<dyn std::error::Error>> {
    let pool = setup_test_db().await;
    let user1 = Uuid::now_v7();
    let user2 = Uuid::now_v7();
    create_test_user(&pool, user1).await?;
    create_test_user(&pool, user2).await?;

    let parent_id = create_message(&pool, user1, user2, "Hello", None, None)
        .await?
        .unwrap();

    create_test_message(&pool, user2, user1, "Hi!", "sig2", Some(parent_id), false).await?;

    let threads = get_user_threads(&pool, user1, Some(10)).await?;
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].id, parent_id);

    Ok(())
}

#[tokio::test]
async fn test_user_with_multiple_threads() -> Result<(), Box<dyn std::error::Error>> {
    let pool = setup_test_db().await;
    let user1 = Uuid::now_v7();
    let user2 = Uuid::now_v7();
    create_test_user(&pool, user1).await?;
    create_test_user(&pool, user2).await?;

    for i in 0..3 {
        let parent_id = create_message(&pool, user1, user2, &format!("Msg {i}"), None, None)
            .await?
            .unwrap();
        create_test_message(&pool, user2, user1, "Reply", "sig", Some(parent_id), false).await?;
    }

    let threads = get_user_threads(&pool, user1, Some(10)).await?;
    assert_eq!(threads.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_limit_is_respected() -> Result<(), Box<dyn std::error::Error>> {
    let pool = setup_test_db().await;
    let user1 = Uuid::now_v7();
    let user2 = Uuid::now_v7();
    create_test_user(&pool, user1).await?;
    create_test_user(&pool, user2).await?;

    for _ in 0..5 {
        let parent_id = create_message(&pool, user1, user2, "Thread start", None, None)
            .await?
            .unwrap();
        create_test_message(&pool, user2, user1, "Reply", "sig", Some(parent_id), false).await?;
    }

    let threads = get_user_threads(&pool, user1, Some(3)).await?;
    assert_eq!(threads.len(), 3);

    Ok(())
}

pub async fn create_test_message(
    pool: &SqlitePool,
    sender_id: Uuid,
    recipient_id: Uuid,
    content: &str,
    signature: &str,
    parent_id: Option<i64>,
    is_read: bool,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().timestamp();
    sqlx::query!(
        r#"
        INSERT INTO messages (sender_id, recipient_id, encrypted_content, signature, parent_id, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        sender_id,
        recipient_id,
        content,
        signature,
        parent_id,
        now,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_test_user(pool: &SqlitePool, id: Uuid) -> Result<(), Error> {
    let full_name = rand::random::<FullName>().to_string();

    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);

    let public_key = hex::encode(verifying_key.to_encoded_point(true).as_bytes());
    let public_key_hash = hex::encode(Sha256::digest(&public_key));

    let user = sqlx::query!(
        r#"INSERT INTO users (id, public_key, public_key_hash, username)
         VALUES (?, ?, ?, ?)
         "#,
        id,
        public_key,
        public_key_hash,
        full_name
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_create_db_pool() {
    let pool = create_db_pool().await.unwrap();
    assert!(pool.acquire().await.is_ok());
}
