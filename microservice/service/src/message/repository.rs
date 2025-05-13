use async_trait::async_trait;
use db::{Error as SqlxError, SqlitePool, db as database, models::Message, uuid::Uuid};
use mockall::automock;
use shared::errors::AppError;

#[automock]
#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn insert_message(
        &self,
        sender_id: Uuid,
        recipient_id: Uuid,
        encrypted_content: &str,
        signature: Option<String>,
        parent_id: Option<i64>,
    ) -> Result<Option<i64>, AppError>;

    async fn get_message_by_id(&self, message_id: i64) -> Result<Option<Message>, AppError>;

    async fn get_conversation(
        &self,
        user1_id: Uuid,
        user2_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, AppError>;

    async fn get_thread_replies(
        &self,
        parent_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Message>, AppError>;

    async fn get_complete_thread(
        &self,
        thread_root_id: i64,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, AppError>;

    async fn get_user_threads(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, AppError>;

    async fn mark_message_read(
        pool: &SqlitePool, 
        message_id: i64
    ) -> Result<(), AppError>;
    
    async fn get_unread_messages(
        pool: &SqlitePool, 
        user_id: Uuid,
    ) -> Result<(), AppError>;
}

#[async_trait]
impl MessageRepository for SqlitePool {
    async fn insert_message(
        &self,
        sender_id: Uuid,
        recipient_id: Uuid,
        encrypted_content: &str,
        signature: Option<String>,
        parent_id: Option<i64>,
    ) -> Result<Option<i64>, AppError> {
        Ok(database::create_message(
            self,
            sender_id,
            recipient_id,
            encrypted_content,
            signature.as_deref(),
            parent_id,
        )
        .await?)
    }

    async fn get_message_by_id(&self, message_id: i64) -> Result<Option<Message>, AppError> {
        Ok(database::get_message(self, message_id).await?)
    }

    async fn get_conversation(
        &self,
        user1_id: Uuid,
        user2_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, AppError> {
        Ok(database::get_conversation(self, user1_id, user2_id, limit).await?)
    }

    async fn get_thread_replies(
        &self,
        parent_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Message>, AppError> {
        Ok(database::get_thread_replies(self, parent_id, limit, offset).await?)
    }

    async fn get_complete_thread(
        &self,
        thread_root_id: i64,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, AppError> {
        Ok(database::get_complete_thread(self, thread_root_id, limit).await?)
    }

    async fn get_user_threads(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, AppError> {
        Ok(database::get_user_threads(self, user_id, limit).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use mockall::predicate;

    fn create_test_uuid(id: u128) -> Uuid {
        Uuid::from_u128(id)
    }

    fn create_test_message(
        id: i64,
        sender_id: Uuid,
        recipient_id: Uuid,
        encrypted_content: &str,
        signature: Option<&str>,
        parent_id: Option<i64>,
    ) -> Message {
        Message {
            id,
            sender_id,
            recipient_id,
            encrypted_content: encrypted_content.to_string(),
            signature: signature.map(|s| s.to_string()),
            parent_id,
            created_at: Utc::now(),
            is_read: false,
        }
    }

    #[tokio::test]
    async fn test_insert_message() {
        let mut mock = MockMessageRepository::new();

        let sender_id = create_test_uuid(1);
        let recipient_id = create_test_uuid(2);
        let encrypted_content = "Test encrypted message";
        let signature = Some("test-signature".to_string());
        let parent_id = Some(42);
        let expected_id = 123;

        mock.expect_insert_message()
            .with(
                predicate::eq(sender_id),
                predicate::eq(recipient_id),
                predicate::eq(encrypted_content),
                predicate::eq(signature.clone()),
                predicate::eq(parent_id),
            )
            .times(1)
            .returning(move |_, _, _, _, _| Ok(Some(expected_id.clone())));

        let result = mock
            .insert_message(
                sender_id,
                recipient_id,
                encrypted_content,
                signature,
                parent_id,
            )
            .await
            .unwrap();

        assert_eq!(result, Some(expected_id));
    }

    #[tokio::test]
    async fn test_insert_message_failure() {
        let mut mock = MockMessageRepository::new();

        let sender_id = create_test_uuid(1);
        let recipient_id = create_test_uuid(2);

        mock.expect_insert_message().returning(|_, _, _, _, _| {
            Err(AppError::DatabaseError(db::Error::InvalidArgument(
                "DB error".to_string(),
            )))
        });

        let result = mock
            .insert_message(sender_id, recipient_id, "encrypted_content", None, None)
            .await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, AppError::DatabaseError(_)));
        }
    }

    #[tokio::test]
    async fn test_get_message_by_id() {
        let mut mock = MockMessageRepository::new();

        let message_id = 123;
        let sender_id = create_test_uuid(1);
        let recipient_id = create_test_uuid(2);

        let expected_message = create_test_message(
            message_id,
            sender_id,
            recipient_id,
            "Test content",
            Some("signature"),
            None,
        );

        mock.expect_get_message_by_id()
            .with(predicate::eq(message_id))
            .times(1)
            .returning(move |_| Ok(Some(expected_message.clone())));

        let result = mock.get_message_by_id(message_id).await.unwrap();

        assert!(result.is_some());
        let message = result.unwrap();
        assert_eq!(message.id, message_id);
        assert_eq!(message.sender_id, sender_id);
        assert_eq!(message.recipient_id, recipient_id);
        assert_eq!(message.encrypted_content, "Test content");
        assert_eq!(message.signature, Some("signature".to_string()));
        assert_eq!(message.parent_id, None);
    }

    #[tokio::test]
    async fn test_get_message_by_id_not_found() {
        let mut mock = MockMessageRepository::new();

        mock.expect_get_message_by_id()
            .with(predicate::eq(999))
            .times(1)
            .returning(|_| Ok(None));

        let result = mock.get_message_by_id(999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_message_by_id_error() {
        let mut mock = MockMessageRepository::new();

        mock.expect_get_message_by_id()
            .returning(|_| Err(AppError::DatabaseError(SqlxError::RowNotFound)));

        let result = mock.get_message_by_id(1).await;
        assert!(matches!(
            result,
            Err(AppError::DatabaseError(SqlxError::RowNotFound))
        ));
    }

    #[tokio::test]
    async fn test_get_conversation() {
        let mut mock = MockMessageRepository::new();

        let user1_id = create_test_uuid(1);
        let user2_id = create_test_uuid(2);
        let limit = Some(10);

        let messages = vec![
            create_test_message(1, user1_id, user2_id, "Message 1", None, None),
            create_test_message(2, user2_id, user1_id, "Reply 1", None, None),
            create_test_message(3, user1_id, user2_id, "Message 2", None, None),
        ];

        mock.expect_get_conversation()
            .with(
                predicate::eq(user1_id),
                predicate::eq(user2_id),
                predicate::eq(limit),
            )
            .times(1)
            .returning(move |_, _, _| Ok(messages.clone()));

        let result = mock
            .get_conversation(user1_id, user2_id, limit)
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].encrypted_content, "Message 1");
        assert_eq!(result[1].encrypted_content, "Reply 1");
        assert_eq!(result[2].encrypted_content, "Message 2");
    }

    #[tokio::test]
    async fn test_get_conversation_error() {
        let mut mock = MockMessageRepository::new();

        mock.expect_get_conversation().returning(|_, _, _| {
            Err(AppError::DatabaseError(SqlxError::InvalidArgument(
                "bad SQL".into(),
            )))
        });

        let result = mock
            .get_conversation(create_test_uuid(1), create_test_uuid(2), Some(5))
            .await;

        assert!(result.is_err());
        if let Err(AppError::DatabaseError(SqlxError::InvalidArgument(msg))) = result {
            assert_eq!(msg, "bad SQL");
        } else {
            panic!("Expected query error");
        }
    }

    #[tokio::test]
    async fn test_get_thread_replies() {
        let mut mock = MockMessageRepository::new();

        let parent_id = 42;
        let limit = Some(5);
        let offset = Some(0);

        let user1_id = create_test_uuid(1);
        let user2_id = create_test_uuid(2);

        let replies = vec![
            create_test_message(43, user2_id, user1_id, "Reply 1", None, Some(parent_id)),
            create_test_message(44, user1_id, user2_id, "Reply 2", None, Some(parent_id)),
        ];

        mock.expect_get_thread_replies()
            .with(
                predicate::eq(parent_id),
                predicate::eq(limit),
                predicate::eq(offset),
            )
            .times(1)
            .returning(move |_, _, _| Ok(replies.clone()));

        let result = mock
            .get_thread_replies(parent_id, limit, offset)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 43);
        assert_eq!(result[1].id, 44);
        assert_eq!(result[0].parent_id, Some(parent_id));
        assert_eq!(result[1].parent_id, Some(parent_id));
    }

    #[tokio::test]
    async fn test_get_thread_replies_invalid_argument() {
        let mut mock = MockMessageRepository::new();

        mock.expect_get_thread_replies().returning(|_, _, _| {
            Err(AppError::DatabaseError(SqlxError::InvalidArgument(
                "invalid parent_id".into(),
            )))
        });

        let result = mock.get_thread_replies(0, Some(10), None).await;
        assert!(matches!(
            result,
            Err(AppError::DatabaseError(SqlxError::InvalidArgument(_)))
        ));
    }

    #[tokio::test]
    async fn test_get_complete_thread() {
        let mut mock = MockMessageRepository::new();

        let thread_root_id = 100;
        let limit = Some(10);

        let user1_id = create_test_uuid(1);
        let user2_id = create_test_uuid(2);

        let thread = vec![
            create_test_message(100, user1_id, user2_id, "Root message", None, None),
            create_test_message(101, user2_id, user1_id, "Reply 1", None, Some(100)),
            create_test_message(102, user1_id, user2_id, "Reply 2", None, Some(100)),
        ];

        mock.expect_get_complete_thread()
            .with(predicate::eq(thread_root_id), predicate::eq(limit))
            .times(1)
            .returning(move |_, _| Ok(thread.clone()));

        let result = mock
            .get_complete_thread(thread_root_id, limit)
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, 100);
        assert_eq!(result[0].parent_id, None);
        assert_eq!(result[1].parent_id, Some(100));
        assert_eq!(result[2].parent_id, Some(100));
    }

    #[tokio::test]
    async fn test_get_complete_thread_timeout() {
        let mut mock = MockMessageRepository::new();

        mock.expect_get_complete_thread()
            .returning(|_, _| Err(AppError::DatabaseError(SqlxError::PoolTimedOut)));

        let result = mock.get_complete_thread(123, Some(5)).await;

        assert!(matches!(
            result,
            Err(AppError::DatabaseError(SqlxError::PoolTimedOut))
        ));
    }

    #[tokio::test]
    async fn test_get_user_threads() {
        let mut mock = MockMessageRepository::new();

        let user_id = create_test_uuid(1);
        let limit = Some(5);

        let user1_id = create_test_uuid(1);
        let user2_id = create_test_uuid(2);

        let threads = vec![
            create_test_message(200, user1_id, user2_id, "Thread 1", None, None),
            create_test_message(201, user1_id, user2_id, "Thread 2", None, None),
        ];

        mock.expect_get_user_threads()
            .with(predicate::eq(user_id.clone()), predicate::eq(limit))
            .times(1)
            .returning(move |_, _| Ok(threads.clone()));

        let result = mock.get_user_threads(user_id, limit).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 200);
        assert_eq!(result[1].id, 201);
    }

    #[tokio::test]
    async fn test_get_conversation_no_limit() {
        let mut mock = MockMessageRepository::new();

        let user1_id = create_test_uuid(1);
        let user2_id = create_test_uuid(2);

        let messages = vec![
            create_test_message(1, user1_id, user2_id, "Msg A", None, None),
            create_test_message(2, user2_id, user1_id, "Msg B", None, None),
        ];

        mock.expect_get_conversation()
            .with(
                predicate::eq(user1_id),
                predicate::eq(user2_id),
                predicate::eq(None),
            )
            .times(1)
            .returning(move |_, _, _| Ok(messages.clone()));

        let result = mock
            .get_conversation(user1_id, user2_id, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_get_thread_replies_no_limit_offset() {
        let mut mock = MockMessageRepository::new();

        let parent_id = 50;
        let user1_id = create_test_uuid(1);
        let user2_id = create_test_uuid(2);

        let replies = vec![create_test_message(
            51,
            user2_id,
            user1_id,
            "Reply X",
            None,
            Some(parent_id),
        )];

        mock.expect_get_thread_replies()
            .with(
                predicate::eq(parent_id),
                predicate::eq(None),
                predicate::eq(None),
            )
            .times(1)
            .returning(move |_, _, _| Ok(replies.clone()));

        let result = mock
            .get_thread_replies(parent_id, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].parent_id, Some(parent_id));
    }

    #[tokio::test]
    async fn test_get_user_threads_empty() {
        let mut mock = MockMessageRepository::new();

        let user_id = create_test_uuid(999);

        mock.expect_get_user_threads()
            .with(predicate::eq(user_id), predicate::eq(Some(10)))
            .times(1)
            .returning(|_, _| Ok(vec![]));

        let result = mock.get_user_threads(user_id, Some(10)).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_user_threads_error() {
        let mut mock = MockMessageRepository::new();

        mock.expect_get_user_threads()
            .returning(|_, _| Err(AppError::DatabaseError(db::Error::WorkerCrashed)));

        let result = mock.get_user_threads(create_test_uuid(100), Some(10)).await;
        assert!(matches!(
            result,
            Err(AppError::DatabaseError(db::Error::WorkerCrashed))
        ));
    }

    #[tokio::test]
    async fn test_get_complete_thread_failure() {
        let mut mock = MockMessageRepository::new();

        mock.expect_get_complete_thread()
            .returning(|_, _| Err(AppError::DatabaseError(db::Error::RowNotFound)));

        let result = mock.get_complete_thread(404, Some(10)).await;
        assert!(result.is_err());
    }
}
