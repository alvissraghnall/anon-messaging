use super::*;
use serial_test::serial;
use sqlx::migrate::MigrateDatabase;
use sqlx::Row;
use std::env;
use std::path::PathBuf;
use std::sync::Once;

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
        CREATE TABLE IF NOT EXISTS revoked_tokens (
            token_hash TEXT PRIMARY KEY NOT NULL,
            revoked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            reason TEXT
        );
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            token_hash TEXT NOT NULL,
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
async fn test_create_db_pool() {
    let pool = create_db_pool().await.unwrap();
    assert!(pool.acquire().await.is_ok());
}

#[tokio::test]
#[serial]
async fn test_insert_user() {
    let pool = setup_test_db().await;
    let public_key_hash = "test_hash";
    let public_key = "encrypted_key";
    let username = "salt";

    let result = insert_user(&pool, public_key_hash, public_key, username).await;
    assert!(result.is_ok());

    let user = get_user_by_id(
        &pool,
        Uuid::parse_str(&result.unwrap()).expect("Was not expected to err"),
    )
    .await
    .unwrap();
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
    let user_id_uuid = Uuid::parse_str(user_id.as_str()).unwrap();
    let user = get_user_by_id(&pool, user_id_uuid).await.unwrap();
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
    assert_eq!(user.created_at, deserialized.created_at);
    assert_eq!(user.updated_at, deserialized.updated_at);
    assert_eq!(user.id, deserialized.id);
    assert_eq!(user.last_login, deserialized.last_login);
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
async fn test_create_message() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let sender_id = Uuid::new_v4().to_string();
    let recipient_id = Uuid::new_v4().to_string();

    create_test_user(&pool, &sender_id).await;
    create_test_user(&pool, &recipient_id).await;

    let encrypted_content = "encrypted test message content";
    let signature = Some("test signature");
    let parent_id = None;

    let message_id = create_message(
        &pool,
        &sender_id,
        &recipient_id,
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

    let sender_id = Uuid::new_v4().to_string();
    let recipient_id = Uuid::new_v4().to_string();
    create_test_user(&pool, &sender_id).await;
    create_test_user(&pool, &recipient_id).await;
    let encrypted_content = "test message content";
    let signature = Some("test signature");
    let parent_id = None;

    let message_id = create_message(
        &pool,
        &sender_id,
        &recipient_id,
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

    let sender_id = Uuid::new_v4().to_string();
    let recipient_id = Uuid::new_v4().to_string();
    create_test_user(&pool, &sender_id).await;
    create_test_user(&pool, &recipient_id).await;
    let encrypted_content = "parent message";

    // Create parent message
    let parent_id = create_message(
        &pool,
        &sender_id,
        &recipient_id,
        encrypted_content,
        None,
        None,
    )
    .await?
    .unwrap();

    // Create child message referencing the parent
    let child_message_id = create_message(
        &pool,
        &recipient_id,
        &sender_id,
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

    let sender_id = Uuid::new_v4().to_string();
    let recipient_id = Uuid::new_v4().to_string();
    let encrypted_content = "test message with invalid parent";

    let result = create_message(
        &pool,
        &sender_id,
        &recipient_id,
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

    let sender_id = Uuid::new_v4().to_string();
    let recipient_id = Uuid::new_v4().to_string();

    create_test_user(&pool, &sender_id).await;
    create_test_user(&pool, &recipient_id).await;
    let message_id = create_message(
        &pool,
        &sender_id,
        &recipient_id,
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

    let user1_id = Uuid::new_v4().to_string();
    let user2_id = Uuid::new_v4().to_string();

    create_test_user(&pool, &user1_id).await;
    create_test_user(&pool, &user2_id).await;
    let messages = get_conversation(&pool, &user1_id, &user2_id, None).await?;

    assert!(messages.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_conversation_with_messages() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let user1_id = Uuid::new_v4().to_string();
    let user2_id = Uuid::new_v4().to_string();
    let user3_id = Uuid::new_v4().to_string();

    create_test_user(&pool, &user1_id).await;
    create_test_user(&pool, &user2_id).await;
    create_test_user(&pool, &user3_id).await;

    let msg1_id = create_message(
        &pool,
        &user1_id,
        &user2_id,
        "Message 1 from user1 to user2",
        None,
        None,
    )
    .await?
    .unwrap();

    let msg2_id = create_message(
        &pool,
        &user2_id,
        &user1_id,
        "Message 2 from user2 to user1",
        None,
        None,
    )
    .await?
    .unwrap();

    create_message(
        &pool,
        &user1_id,
        &user3_id,
        "Message not in conversation",
        None,
        None,
    )
    .await?
    .unwrap();

    let messages = get_conversation(&pool, &user1_id, &user2_id, None).await?;

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

    let user1_id = Uuid::new_v4().to_string();
    let user2_id = Uuid::new_v4().to_string();
    create_test_user(&pool, &user1_id).await;
    create_test_user(&pool, &user2_id).await;

    for i in 0..5 {
        create_message(
            &pool,
            &user1_id,
            &user2_id,
            &format!("Message {}", i),
            None,
            None,
        )
        .await?
        .unwrap();

        // small delay to ensure different created_at timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let messages = get_conversation(&pool, &user1_id, &user2_id, Some(3)).await?;

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

    let user1_id = Uuid::new_v4().to_string();
    let user2_id = Uuid::new_v4().to_string();
    create_test_user(&pool, &user1_id).await;
    create_test_user(&pool, &user2_id).await;

    create_message(
        &pool,
        &user1_id,
        &user2_id,
        "From user1 to user2",
        None,
        None,
    )
    .await?
    .unwrap();

    create_message(
        &pool,
        &user2_id,
        &user1_id,
        "From user2 to user1",
        None,
        None,
    )
    .await?
    .unwrap();

    let messages1 = get_conversation(&pool, &user1_id, &user2_id, None).await?;

    let messages2 = get_conversation(&pool, &user2_id, &user1_id, None).await?;

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

    let user_id = Uuid::new_v4().to_string();
    create_test_user(&pool, &user_id).await;

    // Get unread messages for a user with no messages
    let messages = get_unread_messages(&pool, &user_id).await?;

    assert!(messages.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_unread_messages_mixed() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let recipient_id = Uuid::new_v4().to_string();
    let sender1_id = Uuid::new_v4().to_string();
    let sender2_id = Uuid::new_v4().to_string();
    create_test_user(&pool, &recipient_id).await;
    create_test_user(&pool, &sender1_id).await;
    create_test_user(&pool, &sender2_id).await;

    // Create some unread messages for the recipient
    let msg1_id = create_message(
        &pool,
        &sender1_id,
        &recipient_id,
        "Unread message 1",
        None,
        None,
    )
    .await?
    .unwrap();

    let msg2_id = create_message(
        &pool,
        &sender2_id,
        &recipient_id,
        "Unread message 2",
        None,
        None,
    )
    .await?
    .unwrap();

    // Create a message and mark it as read
    let read_msg_id = create_message(
        &pool,
        &sender1_id,
        &recipient_id,
        "Read message",
        None,
        None,
    )
    .await?
    .unwrap();

    mark_message_read(&pool, read_msg_id).await?;

    // Create a message sent by the recipient (should not be in unread)
    create_message(
        &pool,
        &recipient_id,
        &sender1_id,
        "Message from recipient",
        None,
        None,
    )
    .await?;

    // Get unread messages for the recipient
    let unread_messages = get_unread_messages(&pool, &recipient_id).await?;

    // Should return exactly 2 messages
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

    let recipient_id = Uuid::new_v4().to_string();
    let sender_id = Uuid::new_v4().to_string();
    create_test_user(&pool, &recipient_id).await;
    create_test_user(&pool, &sender_id).await;
    for i in 0..3 {
        create_message(
            &pool,
            &sender_id,
            &recipient_id,
            &format!("Message {}", i),
            None,
            None,
        )
        .await?;

        // Add a small delay to ensure different created_at timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let messages = get_unread_messages(&pool, &recipient_id).await?;

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

    let user1_id = Uuid::new_v4().to_string();
    let user2_id = Uuid::new_v4().to_string();
    create_test_user(&pool, &user1_id).await;
    create_test_user(&pool, &user2_id).await;

    let parent_id = create_message(&pool, &user1_id, &user2_id, "Parent message", None, None)
        .await?
        .unwrap();

    let replies = get_thread_replies(&pool, parent_id, None, None).await?;

    assert!(replies.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_thread_replies_with_replies() -> Result<(), Error> {
    let pool = setup_test_db().await;

    let user1_id = Uuid::new_v4().to_string();
    let user2_id = Uuid::new_v4().to_string();
    create_test_user(&pool, &user1_id).await;
    create_test_user(&pool, &user2_id).await;

    let parent_id = create_message(&pool, &user1_id, &user2_id, "Parent message", None, None)
        .await?
        .unwrap();

    let reply1_id = create_message(
        &pool,
        &user2_id,
        &user1_id,
        "Reply 1",
        None,
        Some(parent_id),
    )
    .await?
    .unwrap();

    // Add a delay to ensure different timestamps
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let reply2_id = create_message(
        &pool,
        &user1_id,
        &user2_id,
        "Reply 2",
        None,
        Some(parent_id),
    )
    .await?
    .unwrap();

    let other_parent_id = create_message(&pool, &user1_id, &user2_id, "Another parent", None, None)
        .await?
        .unwrap();

    let other_reply_id = create_message(
        &pool,
        &user2_id,
        &user1_id,
        "Reply to different parent",
        None,
        Some(other_parent_id),
    )
    .await?
    .unwrap();

    // Get replies to the first thread
    let replies = get_thread_replies(&pool, parent_id, None, None).await?;

    // Should return exactly 2 replies
    assert_eq!(replies.len(), 2);

    // Verify they are the correct replies
    let reply_ids: Vec<i64> = replies.iter().map(|m| m.id).collect();
    assert!(reply_ids.contains(&reply1_id));
    assert!(reply_ids.contains(&reply2_id));
    assert!(!reply_ids.contains(&other_reply_id));

    // Verify all messages have the correct parent_id
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
    // Setup in-memory SQLite database for testing
    let pool = setup_test_db().await;

    let user1_id = Uuid::new_v4().to_string();
    let user2_id = Uuid::new_v4().to_string();
    create_test_user(&pool, &user1_id).await;
    create_test_user(&pool, &user2_id).await;

    let parent_id = create_message(&pool, &user1_id, &user2_id, "Parent message", None, None)
        .await?
        .unwrap();

    let mut reply_ids = Vec::with_capacity(5);
    for i in 0..5 {
        let sender = if i % 2 == 0 { &user1_id } else { &user2_id };
        let recipient = if i % 2 == 0 { &user2_id } else { &user1_id };

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

pub async fn create_test_user(pool: &SqlitePool, user_id: &str) -> Result<(), Error> {
    let id = Uuid::now_v7();
    let user = sqlx::query!(
        r#"INSERT INTO users (id, public_key, public_key_hash, username)
         VALUES (?, ?, ?, ?)
         "#,
        id,
        "public_key",
        "public_key_hash",
        "username"
    )
    .fetch_one(pool)
    .await?;

    Ok(())
}
