use db::{models::Message, uuid::Uuid};
use shared::errors::AppError;

use super::repository::MessageRepository;

#[derive(Clone)]
pub struct MessageService<R: MessageRepository> {
    repository: R,
}

impl<R: MessageRepository> MessageService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_message_by_id(&self, message_id: i64) -> Result<Option<Message>, AppError> {
        self.repository.get_message_by_id(message_id).await
    }

    pub async fn create_message(
        &self,
        sender_id: Uuid,
        recipient_id: Uuid,
        encrypted_content: &str,
        signature: Option<String>,
        parent_id: Option<i64>,
    ) -> Result<Option<i64>, AppError> {
        self.repository
            .insert_message(
                sender_id,
                recipient_id,
                encrypted_content,
                signature,
                parent_id,
            )
            .await
    }

    pub async fn get_conversation(
        &self,
        user1_id: Uuid,
        user2_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, AppError> {
        self.repository
            .get_conversation(user1_id, user2_id, limit)
            .await
    }

    pub async fn get_thread_replies(
        &self,
        parent_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Message>, AppError> {
        self.repository
            .get_thread_replies(parent_id, limit, offset)
            .await
    }

    pub async fn get_complete_thread(
        &self,
        thread_root_id: i64,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, AppError> {
        self.repository
            .get_complete_thread(thread_root_id, limit)
            .await
    }

    pub async fn get_user_threads(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Message>, AppError> {
        self.repository.get_user_threads(user_id, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use db::uuid::Uuid;
    use mockall::mock;
    use mockall::predicate::*;
    use shared::errors::AppError;
    use std::sync::Arc;

    mock! {
        Repository {}

        #[async_trait]
        impl MessageRepository for Repository {
            async fn get_message_by_id(&self, message_id: i64) -> Result<Option<Message>, AppError>;
            async fn insert_message(
                &self,
                sender_id: Uuid,
                recipient_id: Uuid,
                encrypted_content: &str,
                signature: Option<String>,
                parent_id: Option<i64>,
            ) -> Result<Option<i64>, AppError>;
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
                &self,
                message_id: i64
            ) -> Result<(), AppError>;

            async fn get_unread_messages(
                &self,
                user_id: Uuid,
            ) -> Result<Vec<Message>, AppError>;
        }
    }

    fn create_test_message(id: i64) -> Message {
        Message {
            id,
            sender_id: Uuid::new_v4(),
            recipient_id: Uuid::new_v4(),
            encrypted_content: "Test content".to_string(),
            signature: Some("Test signature".to_string()),
            parent_id: None,
            created_at: chrono::Utc::now(),
            is_read: false,
        }
    }

    #[tokio::test]
    async fn test_get_message_by_id_success() {
        let mut mock_repo = MockRepository::new();
        let message_id = 1;
        let expected_message = create_test_message(message_id);

        mock_repo
            .expect_get_message_by_id()
            .with(eq(message_id))
            .times(1)
            .returning(move |_| Ok(Some(expected_message.clone())));

        let service = MessageService::new(mock_repo);
        let result = service.get_message_by_id(message_id).await.unwrap();

        assert!(result.is_some());
        let message = result.unwrap();
        assert_eq!(message.id, message_id);
    }

    #[tokio::test]
    async fn test_get_message_by_id_not_found() {
        let mut mock_repo = MockRepository::new();
        let message_id = 999;

        mock_repo
            .expect_get_message_by_id()
            .with(eq(message_id))
            .times(1)
            .returning(|_| Ok(None));

        let service = MessageService::new(mock_repo);
        let result = service.get_message_by_id(message_id).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_message_by_id_error() {
        let mut mock_repo = MockRepository::new();
        let message_id = 1;

        mock_repo
            .expect_get_message_by_id()
            .with(eq(message_id))
            .times(1)
            .returning(|_| {
                Err(AppError::DatabaseError(db::Error::InvalidArgument(
                    "DB error".to_string(),
                )))
            });

        let service = MessageService::new(mock_repo);
        let result = service.get_message_by_id(message_id).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_message_success() {
        let mut mock_repo = MockRepository::new();
        let sender_id = Uuid::now_v7();
        let recipient_id = Uuid::now_v7();
        let encrypted_content = "encrypted test message";
        let signature = Some("test signature".to_string());
        let parent_id = None;
        let expected_id = 1;

        mock_repo
            .expect_insert_message()
            .with(
                eq(sender_id),
                eq(recipient_id),
                eq(encrypted_content),
                eq(signature.clone()),
                eq(parent_id),
            )
            .times(1)
            .returning(move |_, _, _, _, _| Ok(Some(expected_id)));

        let service = MessageService::new(mock_repo);
        let result = service
            .create_message(
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
    async fn test_create_message_error() {
        let mut mock_repo = MockRepository::new();
        let sender_id = Uuid::now_v7();
        let recipient_id = Uuid::now_v7();
        let encrypted_content = "encrypted test message";
        let signature = Some("test signature".to_string());
        let parent_id = None;

        mock_repo
            .expect_insert_message()
            .with(
                eq(sender_id),
                eq(recipient_id),
                eq(encrypted_content),
                eq(signature.clone()),
                eq(parent_id),
            )
            .times(1)
            .returning(|_, _, _, _, _| {
                Err(AppError::DatabaseError(db::Error::InvalidArgument(
                    "DB error".to_string(),
                )))
            });

        let service = MessageService::new(mock_repo);
        let result = service
            .create_message(
                sender_id,
                recipient_id,
                encrypted_content,
                signature,
                parent_id,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_message_with_empty_content() {
        let mut mock_repo = MockRepository::new();
        let sender_id = Uuid::now_v7();
        let recipient_id = Uuid::now_v7();
        let encrypted_content = "";
        let signature = None;
        let parent_id = None;

        mock_repo
            .expect_insert_message()
            .with(
                eq(sender_id),
                eq(recipient_id),
                eq(encrypted_content),
                eq(signature.clone()),
                eq(parent_id),
            )
            .times(1)
            .returning(|_, _, _, _, _| Ok(Some(1)));

        let service = MessageService::new(mock_repo);
        let result = service
            .create_message(
                sender_id,
                recipient_id,
                encrypted_content,
                signature,
                parent_id,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_conversation_success() {
        let mut mock_repo = MockRepository::new();
        let user1_id = Uuid::now_v7();
        let user2_id = Uuid::now_v7();
        let limit = Some(10);
        let expected_messages = vec![
            create_test_message(1),
            create_test_message(2),
            create_test_message(3),
        ];

        mock_repo
            .expect_get_conversation()
            .with(eq(user1_id), eq(user2_id), eq(limit))
            .times(1)
            .returning(move |_, _, _| Ok(expected_messages.clone()));

        let service = MessageService::new(mock_repo);
        let result = service
            .get_conversation(user1_id, user2_id, limit)
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_get_conversation_empty() {
        let mut mock_repo = MockRepository::new();
        let user1_id = Uuid::now_v7();
        let user2_id = Uuid::now_v7();
        let limit = Some(10);

        mock_repo
            .expect_get_conversation()
            .with(eq(user1_id), eq(user2_id), eq(limit))
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let service = MessageService::new(mock_repo);
        let result = service
            .get_conversation(user1_id, user2_id, limit)
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_conversation_with_pagination() {
        let mut mock_repo = MockRepository::new();
        let user1_id = Uuid::now_v7();
        let user2_id = Uuid::now_v7();
        let limit = Some(5);
        let expected_messages = vec![create_test_message(1), create_test_message(2)];

        mock_repo
            .expect_get_conversation()
            .with(eq(user1_id), eq(user2_id), eq(limit))
            .times(1)
            .returning(move |_, _, _| Ok(expected_messages.clone()));

        let service = MessageService::new(mock_repo);
        let result = service
            .get_conversation(user1_id, user2_id, limit)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|m| m.id == 1 || m.id == 2));
    }

    #[tokio::test]
    async fn test_get_thread_replies_success() {
        let mut mock_repo = MockRepository::new();
        let parent_id = 1;
        let limit = Some(10);
        let offset = Some(0);
        let expected_replies = vec![create_test_message(2), create_test_message(3)];

        mock_repo
            .expect_get_thread_replies()
            .with(eq(parent_id), eq(limit), eq(offset))
            .times(1)
            .returning(move |_, _, _| Ok(expected_replies.clone()));

        let service = MessageService::new(mock_repo);
        let result = service
            .get_thread_replies(parent_id, limit, offset)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_get_complete_thread_success() {
        let mut mock_repo = MockRepository::new();
        let thread_root_id = 1;
        let limit = Some(20);
        let expected_thread = vec![
            create_test_message(1),
            create_test_message(2),
            create_test_message(3),
            create_test_message(4),
        ];

        mock_repo
            .expect_get_complete_thread()
            .with(eq(thread_root_id), eq(limit))
            .times(1)
            .returning(move |_, _| Ok(expected_thread.clone()));

        let service = MessageService::new(mock_repo);
        let result = service
            .get_complete_thread(thread_root_id, limit)
            .await
            .unwrap();

        assert_eq!(result.len(), 4);
    }

    #[tokio::test]
    async fn test_get_thread_replies_with_offset() {
        let mut mock_repo = MockRepository::new();
        let parent_id = 1;
        let limit = Some(2);
        let offset = Some(2);
        let expected_replies = vec![create_test_message(3), create_test_message(4)];

        mock_repo
            .expect_get_thread_replies()
            .with(eq(parent_id), eq(limit), eq(offset))
            .times(1)
            .returning(move |_, _, _| Ok(expected_replies.clone()));

        let service = MessageService::new(mock_repo);
        let result = service
            .get_thread_replies(parent_id, limit, offset)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|m| m.id == 3 || m.id == 4));
    }

    #[tokio::test]
    async fn test_create_message_with_parent_id() {
        let mut mock_repo = MockRepository::new();
        let sender_id = Uuid::now_v7();
        let recipient_id = Uuid::now_v7();
        let encrypted_content = "reply to parent";
        let signature = None;
        let parent_id = Some(1);

        mock_repo
            .expect_insert_message()
            .with(
                eq(sender_id),
                eq(recipient_id),
                eq(encrypted_content),
                eq(signature.clone()),
                eq(parent_id),
            )
            .times(1)
            .returning(|_, _, _, _, _| Ok(Some(2)));

        let service = MessageService::new(mock_repo);
        let result = service
            .create_message(
                sender_id,
                recipient_id,
                encrypted_content.clone(),
                signature,
                parent_id,
            )
            .await
            .unwrap();

        assert_eq!(result, Some(2));
    }

    #[tokio::test]
    async fn test_deep_thread_retrieval() {
        let mut mock_repo = MockRepository::new();
        let thread_root_id = 1;
        let limit = Some(50);
        let mut expected_thread = Vec::new();

        for i in 1..=50 {
            expected_thread.push(create_test_message(i));
        }

        mock_repo
            .expect_get_complete_thread()
            .with(eq(thread_root_id), eq(limit))
            .times(1)
            .returning(move |_, _| Ok(expected_thread.clone()));

        let service = MessageService::new(mock_repo);
        let result = service
            .get_complete_thread(thread_root_id, limit)
            .await
            .unwrap();

        assert_eq!(result.len(), 50);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[49].id, 50);
    }

    #[tokio::test]
    async fn test_create_message_with_large_content() {
        let mut mock_repo = MockRepository::new();
        let sender_id = Uuid::now_v7();
        let recipient_id = Uuid::now_v7();
        let encrypted_content = "a".repeat(10_000);
        let signature = None;
        let parent_id = None;

        mock_repo
            .expect_insert_message()
            .with(
                eq(sender_id),
                eq(recipient_id),
                eq(encrypted_content.clone()),
                eq(signature.clone()),
                eq(parent_id),
            )
            .times(1)
            .returning(|_, _, _, _, _| Ok(Some(1)));

        let service = MessageService::new(mock_repo);
        let result = service
            .create_message(
                sender_id,
                recipient_id,
                &encrypted_content,
                signature,
                parent_id,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_threads_success() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::now_v7();
        let limit = Some(10);
        let expected_threads = vec![
            create_test_message(1),
            create_test_message(5),
            create_test_message(10),
        ];

        mock_repo
            .expect_get_user_threads()
            .with(eq(user_id.clone()), eq(limit))
            .times(1)
            .returning(move |_, _| Ok(expected_threads.clone()));

        let service = MessageService::new(mock_repo);
        let result = service.get_user_threads(user_id, limit).await.unwrap();

        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_get_user_threads_empty() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::now_v7();
        let user_id_str = user_id.to_string();
        let limit = Some(10);

        mock_repo
            .expect_get_user_threads()
            .with(eq(user_id.clone()), eq(limit))
            .times(1)
            .returning(|_, _| Ok(vec![]));

        let service = MessageService::new(mock_repo);
        let result = service.get_user_threads(user_id, limit).await.unwrap();

        assert!(result.is_empty());
    }
}
