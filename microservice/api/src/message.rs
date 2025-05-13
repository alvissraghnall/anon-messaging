use actix_web::{
    get, post,
    web::{self, Data, Json, Path, Query},
    HttpResponse, Responder,
};
use base64::Engine;
use db::{models::Message, uuid::Uuid};
use mockall::automock;
use service::message::{repository::MessageRepository, service::MessageService};
use shared::{
    errors::AppError,
    models::{
        CreateMessageRequest, CreateMessageResponse as MessageCreatedResponse, CUSTOM_ENGINE,
    },
};
use std::sync::Arc;
use utoipa::ToSchema;
use validator::Validate;
use serde::Deserialize;

pub type CreateMessageResponse = Result<HttpResponse, AppError>;
pub type GetMessageResponse = Result<HttpResponse, AppError>;
pub type GetConversationResponse = Result<HttpResponse, AppError>;
pub type GetThreadRepliesResponse = Result<HttpResponse, AppError>;
pub type GetCompleteThreadResponse = Result<HttpResponse, AppError>;
pub type GetUserThreadsResponse = Result<HttpResponse, AppError>;


#[derive(Deserialize)]
struct LimitQuery {
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
struct LimitAndOffsetQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>
}

#[automock]
#[async_trait::async_trait]
pub trait MessageController: Send + Sync {
    async fn create_message(&self, request: Json<CreateMessageRequest>) -> CreateMessageResponse;

    async fn get_message(&self, message_id: Path<i64>) -> GetMessageResponse;

    async fn get_conversation(
        &self,
        user1_id: Path<Uuid>,
        user2_id: Path<Uuid>,
        limit: Query<Option<i64>>,
    ) -> GetConversationResponse;

    async fn get_thread_replies(
        &self,
        parent_id: Path<i64>,
        limit: Query<Option<i64>>,
        offset: Query<Option<i64>>,
    ) -> GetThreadRepliesResponse;

    async fn get_complete_thread(
        &self,
        thread_root_id: Path<i64>,
        limit: Query<Option<i64>>,
    ) -> GetCompleteThreadResponse;

    async fn get_user_threads(
        &self,
        user_id: Path<Uuid>,
        limit: Query<Option<i64>>,
    ) -> GetUserThreadsResponse;
}

pub struct MessageControllerImpl<R: MessageRepository> {
    service: Data<MessageService<R>>,
}

impl<R: MessageRepository> MessageControllerImpl<R> {
    pub fn new(service: Data<MessageService<R>>) -> Self {
        Self { service }
    }
}

#[async_trait::async_trait]
impl<R: MessageRepository + 'static> MessageController for MessageControllerImpl<R> {
    async fn create_message(&self, request: Json<CreateMessageRequest>) -> CreateMessageResponse {
        if let Err(validation_errors) = request.validate() {
            return Err(AppError::ValidationError(validation_errors));
        }
        let message_id = self
            .service
            .create_message(
                request.sender_id,
                request.recipient_id,
                &request.encrypted_content,
                request.signature.clone(),
                request.parent_id,
            )
            .await?
            .ok_or_else(|| AppError::InternalError(String::from("Failed to create message")))?;

        Ok(HttpResponse::Created().json(MessageCreatedResponse::new(message_id)))
    }

    async fn get_message(&self, message_id: Path<i64>) -> GetMessageResponse {
        let message = self
            .service
            .get_message_by_id(*message_id)
            .await?
            .ok_or_else(|| AppError::NotFound(String::from("Message not found")))?;

        Ok(HttpResponse::Ok().json(message))
    }

    async fn get_conversation(
        &self,
        user1_id: Path<Uuid>,
        user2_id: Path<Uuid>,
        limit: Query<Option<i64>>,
    ) -> GetConversationResponse {
        let messages = self
            .service
            .get_conversation(*user1_id, *user2_id, limit.into_inner())
            .await?;

        Ok(HttpResponse::Ok().json(messages))
    }

    async fn get_thread_replies(
        &self,
        parent_id: Path<i64>,
        limit: Query<Option<i64>>,
        offset: Query<Option<i64>>,
    ) -> GetThreadRepliesResponse {
        let replies = self
            .service
            .get_thread_replies(*parent_id, limit.into_inner(), offset.into_inner())
            .await?;

        Ok(HttpResponse::Ok().json(replies))
    }

    async fn get_complete_thread(
        &self,
        thread_root_id: Path<i64>,
        limit: Query<Option<i64>>,
    ) -> GetCompleteThreadResponse {
        let thread = self
            .service
            .get_complete_thread(*thread_root_id, limit.into_inner())
            .await?;

        Ok(HttpResponse::Ok().json(thread))
    }

    async fn get_user_threads(
        &self,
        user_id: Path<Uuid>,
        limit: Query<Option<i64>>,
    ) -> GetUserThreadsResponse {
        let threads = self
            .service
            .get_user_threads(*user_id, limit.into_inner())
            .await?;

        Ok(HttpResponse::Ok().json(threads))
    }
}

// Actix-web route handlers
#[utoipa::path(
    post,
    path = "/api/messages",
    request_body(content = CreateMessageRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Message created successfully", body = MessageCreatedResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error"),
    )
)]
#[post("")]
pub async fn create_message_handler(
    controller: Data<Arc<dyn MessageController>>,
    request: Json<CreateMessageRequest>,
) -> impl Responder {
    controller.create_message(request).await
}

#[utoipa::path(
    get,
    path = "/api/messages/{message_id}",
    params(
        ("message_id" = i64, Path, description = "Message ID")
    ),
    responses(
        (status = 200, description = "Message found", body = Message),
        (status = 404, description = "Message not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[get("/{message_id}")]
pub async fn get_message_handler(
    controller: Data<Arc<dyn MessageController>>,
    message_id: Path<i64>,
) -> impl Responder {
    controller.get_message(message_id).await
}

#[utoipa::path(
    get,
    path = "/api/messages/conversations/{user1_id}/{user2_id}",
    params(
        ("user1_id" = Uuid, Path, description = "First user ID"),
        ("user2_id" = Uuid, Path, description = "Second user ID"),
        ("limit" = Option<i64>, Query, description = "Maximum number of messages to return")
    ),
    responses(
        (status = 200, description = "Conversation messages", body = Vec<Message>),
        (status = 500, description = "Internal server error"),
    )
)]
#[get("/conversations/{user1_id}/{user2_id}")]
pub async fn get_conversation_handler(
    controller: Data<Arc<dyn MessageController>>,
    path: Path<(Uuid, Uuid)>,
    limit: Query<LimitQuery>,
) -> impl Responder {
    let (user1_id, user2_id) = path.into_inner();
    controller
        .get_conversation(Path::from(user1_id), Path::from(user2_id), Query(limit.limit))
        .await
}

#[utoipa::path(
    get,
    path = "/api/messages/threads/{parent_id}/replies",
    params(
        ("parent_id" = i64, Path, description = "Parent message ID"),
        ("limit" = Option<i64>, Query, description = "Maximum number of replies to return"),
        ("offset" = Option<i64>, Query, description = "Number of replies to skip")
    ),
    responses(
        (status = 200, description = "Thread replies", body = Vec<Message>),
        (status = 500, description = "Internal server error"),
    )
)]
#[get("/threads/{parent_id}/replies")]
pub async fn get_thread_replies_handler(
    controller: Data<Arc<dyn MessageController>>,
    parent_id: Path<i64>,
    query: Query<LimitAndOffsetQuery>,
) -> impl Responder {
    controller
        .get_thread_replies(parent_id, Query(query.limit), Query(query.offset))
        .await
}

#[utoipa::path(
    get,
    path = "/api/messages/threads/{thread_root_id}",
    params(
        ("thread_root_id" = i64, Path, description = "Thread root message ID"),
        ("limit" = Option<i64>, Query, description = "Maximum number of messages to return")
    ),
    responses(
        (status = 200, description = "Complete thread", body = Vec<Message>),
        (status = 500, description = "Internal server error"),
    )
)]
#[get("/threads/{thread_root_id}")]
pub async fn get_complete_thread_handler(
    controller: Data<Arc<dyn MessageController>>,
    thread_root_id: Path<i64>,
    query: Query<LimitQuery>,
) -> impl Responder {
    controller.get_complete_thread(thread_root_id, Query(query.limit)).await
}

#[utoipa::path(
    get,
    path = "/api/messages/users/{user_id}/threads",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ("limit" = Option<i64>, Query, description = "Maximum number of threads to return")
    ),
    responses(
        (status = 200, description = "User threads", body = Vec<Message>),
        (status = 500, description = "Internal server error"),
    )
)]
#[get("/users/{user_id}/threads")]
pub async fn get_user_threads_handler(
    controller: Data<Arc<dyn MessageController>>,
    user_id: Path<Uuid>,
    query: Query<LimitQuery>,
) -> impl Responder {
    controller.get_user_threads(user_id, Query(query.limit)).await
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/messages")
            .service(create_message_handler)
            .service(get_message_handler)
            .service(get_conversation_handler)
            .service(get_thread_replies_handler)
            .service(get_complete_thread_handler)
            .service(get_user_threads_handler),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{body::to_bytes, test};
    use mockall::predicate::*;
    use sqlx::types::chrono::Utc;
    use validator::{ValidationError, ValidationErrors};
    use mockall::mock;
    // use service::message::repository::MockMessageRepository;

    mock! {
        Repository {}

        #[async_trait::async_trait]
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
        }
    }

    impl Clone for MockRepository {
        fn clone(&self) -> Self {
            Self::default()
        }
    }

    async fn setup_controller() -> (
        Data<MessageControllerImpl<MockRepository>>,
        MockRepository,
    ) {
        let mock_repo = MockRepository::new();
        let service = Data::new(MessageService::new(mock_repo.clone()));
        let controller = Data::new(MessageControllerImpl::new(service));
        (controller, mock_repo)
    }

    impl Clone for MockMessageController {
        fn clone(&self) -> Self {
            Self::default()
        }
    }

    async fn parse_response_body<T: serde::de::DeserializeOwned>(response: HttpResponse) -> T {
        let body = to_bytes(response.into_body()).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[actix_web::test]
    async fn test_create_message_success() {
        let mut mock_repo = MockRepository::new();

        let encrypted_content = CUSTOM_ENGINE.encode(b"abcd");
        let sig = CUSTOM_ENGINE.encode(b"signature");

        let message_id = 42;
        let request = CreateMessageRequest {
            sender_id: Uuid::now_v7(),
            recipient_id: Uuid::now_v7(),
            encrypted_content: encrypted_content.clone(),
            signature: Some(sig.clone()),
            parent_id: None,
        };

        mock_repo
            .expect_insert_message()
            .times(1)
            .returning(move |_, _, _, _, _| Ok(Some(message_id)));
            
        let service = Data::new(MessageService::new(mock_repo));
        let controller = Data::new(MessageControllerImpl::new(service));
       
        let response = controller.create_message(Json(request)).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body: MessageCreatedResponse = parse_response_body(response).await;
        assert_eq!(body.get_message_id(), message_id);
    }

    #[actix_web::test]
    async fn test_create_message_success_2() {
        let mut mock_repo = MockRepository::new();

        let message_id = 123;

        mock_repo
            .expect_insert_message()
            .times(1)
            .returning(move |_, _, _, _, _| Ok(Some(message_id)));
            
        let service = Data::new(MessageService::new(mock_repo));
        let controller = Data::new(MessageControllerImpl::new(service));
       
        let request = CreateMessageRequest {
            sender_id: Uuid::now_v7(),
            recipient_id: Uuid::now_v7(),
            encrypted_content: CUSTOM_ENGINE.encode(b"valid"),
            signature: Some(CUSTOM_ENGINE.encode(b"sig")),
            parent_id: None,
        };

        let response = controller.create_message(Json(request)).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body: MessageCreatedResponse = parse_response_body(response).await;
        assert_eq!(body.get_message_id(), message_id);
    }

    #[actix_web::test]
    async fn test_create_message_invalid_encrypted_content() {
        let (controller, _) = setup_controller().await;

        let request = CreateMessageRequest {
            sender_id: Uuid::now_v7(),
            recipient_id: Uuid::now_v7(),
            encrypted_content: "not_base64".to_string(),
            signature: Some("c2lnbmF0dXJl".to_string()),
            parent_id: None,
        };

        let response = controller.create_message(Json(request)).await;

        match response {
            Err(AppError::ValidationError(errs)) => {
                assert!(errs.field_errors().contains_key("encrypted_content"));
            }
            _ => panic!("Expected ValidationError for invalid base64 in encrypted_content"),
        }
    }

    #[actix_web::test]
    async fn test_create_message_internal_error() {
        let mut mock_repo = MockRepository::new();

        mock_repo.expect_insert_message()
            .times(1)
            .returning(|_, _, _, _, _| Err(AppError::InternalError(String::from("failed"))));

        let request = CreateMessageRequest {
            sender_id: Uuid::now_v7(),
            recipient_id: Uuid::now_v7(),
            encrypted_content: CUSTOM_ENGINE.encode(b"valid"),
            signature: Some(CUSTOM_ENGINE.encode(b"sig")),
            parent_id: None,
        };
        let service = Data::new(MessageService::new(mock_repo));
        let controller = Data::new(MessageControllerImpl::new(service));

        let result = controller.create_message(Json(request)).await;
        println!("{:?}", result);

        match result {
            Err(AppError::InternalError(_)) => {}
            _ => panic!("Expected InternalError"),
        }
    }

    #[actix_web::test]
    async fn test_create_message_signature_too_long() {
        let (controller, _) = setup_controller().await;

        let long_data = CUSTOM_ENGINE.encode(vec![0u8; 513]);
        let enc_content = CUSTOM_ENGINE.encode(b"abcd");

        let request = CreateMessageRequest {
            sender_id: Uuid::now_v7(),
            recipient_id: Uuid::now_v7(),
            encrypted_content: enc_content.clone(),
            signature: Some(long_data.clone()),
            parent_id: None,
        };

        let response = controller.create_message(Json(request)).await;

        match response {
            Err(AppError::ValidationError(errs)) => {
                assert!(errs.field_errors().contains_key("signature"));
            }
            _ => panic!("Expected ValidationError for signature too long"),
        }
    }

    #[actix_web::test]
    async fn test_get_message_success() {
        let mut mock_repo = MockRepository::new();

        let message_id = 42;
        let test_message = Message {
            id: message_id,
            sender_id: Uuid::now_v7(),
            recipient_id: Uuid::now_v7(),
            encrypted_content: "test message".to_string(),
            signature: Some("test signature".to_string()),
            parent_id: None,
            created_at: Utc::now(),
            is_read: false,
        };

        mock_repo
            .expect_get_message_by_id()
            .with(eq(message_id))
            .times(1)
            .returning(move |_| Ok(Some(test_message.clone())));

        
        let service = Data::new(MessageService::new(mock_repo));
        let controller = Data::new(MessageControllerImpl::new(service));
        let response = controller
            .get_message(Path::from(message_id))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Message = parse_response_body(response).await;
        assert_eq!(body.id, message_id);
        assert_eq!(body.encrypted_content, "test message");
    }

    #[actix_web::test]
    async fn test_get_conversation_success() {
        let mut mock_repo = MockRepository::new();

        let user1_id = Uuid::now_v7();
        let user2_id = Uuid::now_v7();

        let messages = vec![
            Message {
                id: 1,
                sender_id: user1_id,
                recipient_id: user2_id,
                encrypted_content: "message 1".to_string(),
                signature: None,
                parent_id: None,
                created_at: Utc::now(),
                is_read: false,
            },
            Message {
                id: 2,
                sender_id: user2_id,
                recipient_id: user1_id,
                encrypted_content: "message 2".to_string(),
                signature: None,
                parent_id: None,
                created_at: Utc::now(),
                is_read: false,
            },
        ];

        mock_repo.expect_get_conversation()
            .with(
                eq(user1_id),
                eq(user2_id),
                eq(Some(10)),
            )
            .times(1)
            .returning(move |_, _, _| Ok(messages.clone()));

        let service = Data::new(MessageService::new(mock_repo));
        let controller = Data::new(MessageControllerImpl::new(service));
        
        let response = controller
            .get_conversation(Path::from(user1_id), Path::from(user2_id), Query(Some(10)))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Vec<Message> = parse_response_body(response).await;
        assert_eq!(body.len(), 2);
        assert_eq!(body[0].encrypted_content, "message 1");
        assert_eq!(body[1].encrypted_content, "message 2");
    }

    #[actix_web::test]
    async fn test_get_thread_replies_success() {
        let mut mock_repo = MockRepository::new();

        let parent_id = 1;
        let replies = vec![
            Message {
                id: 2,
                sender_id: Uuid::now_v7(),
                recipient_id: Uuid::now_v7(),
                encrypted_content: "reply 1".to_string(),
                signature: None,
                parent_id: Some(parent_id),
                created_at: Utc::now(),
                is_read: false,
            },
            Message {
                id: 3,
                sender_id: Uuid::now_v7(),
                recipient_id: Uuid::now_v7(),
                encrypted_content: "reply 2".to_string(),
                signature: None,
                parent_id: Some(parent_id),
                created_at: Utc::now(),
                is_read: false,
            },
        ];

        mock_repo.expect_get_thread_replies()
            .with(
                eq(parent_id),
                eq(Some(10)),
                eq(Some(0)),
            )
            .times(1)
            .returning(move |_, _, _| Ok(replies.clone()));

        let service = Data::new(MessageService::new(mock_repo));
        let controller = Data::new(MessageControllerImpl::new(service));

        let response = controller
            .get_thread_replies(Path::from(parent_id), Query(Some(10)), Query(Some(0)))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Vec<Message> = parse_response_body(response).await;
        assert_eq!(body.len(), 2);
        assert_eq!(body[0].encrypted_content, "reply 1");
        assert_eq!(body[1].encrypted_content, "reply 2");
    }

    #[actix_web::test]
    async fn test_get_complete_thread_success() {
        let mut mock_repo = MockRepository::new();

        let thread_root_id = 1;
        let thread = vec![
            Message {
                id: thread_root_id,
                sender_id: Uuid::now_v7(),
                recipient_id: Uuid::now_v7(),
                encrypted_content: "root message".to_string(),
                signature: None,
                parent_id: None,
                created_at: Utc::now(),
                is_read: false,
            },
            Message {
                id: 2,
                sender_id: Uuid::now_v7(),
                recipient_id: Uuid::now_v7(),
                encrypted_content: "reply 1".to_string(),
                signature: None,
                parent_id: Some(thread_root_id),
                created_at: Utc::now(),
                is_read: false,
            },
        ];

        mock_repo.expect_get_complete_thread()
            .with(
                eq(thread_root_id),
                eq(Some(10)),
            )
            .times(1)
            .returning(move |_, _| Ok(thread.clone()));

        let service = Data::new(MessageService::new(mock_repo));
        let controller = Data::new(MessageControllerImpl::new(service));
        
        let response = controller
            .get_complete_thread(Path::from(thread_root_id), Query(Some(10)))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Vec<Message> = parse_response_body(response).await;
        assert_eq!(body.len(), 2);
        assert_eq!(body[0].encrypted_content, "root message");
        assert_eq!(body[1].encrypted_content, "reply 1");
    }

    #[actix_web::test]
    async fn test_get_user_threads_success() {
        let mut mock_repo = MockRepository::new();

        let user_id = Uuid::now_v7();
        let threads = vec![
            Message {
                id: 1,
                sender_id: user_id,
                recipient_id: Uuid::now_v7(),
                encrypted_content: "thread 1".to_string(),
                signature: None,
                parent_id: None,
                created_at: Utc::now(),
                is_read: false,
            },
            Message {
                id: 2,
                sender_id: user_id,
                recipient_id: Uuid::now_v7(),
                encrypted_content: "thread 2".to_string(),
                signature: None,
                parent_id: None,
                created_at: Utc::now(),
                is_read: false,
            },
        ];
        
        mock_repo.expect_get_user_threads()
            .with(
                eq(user_id),
                eq(Some(10)),
            )
            .times(1)
            .returning(move |_, _| Ok(threads.clone()));

        let service = Data::new(MessageService::new(mock_repo));
        let controller = Data::new(MessageControllerImpl::new(service));
        
        let response = controller
            .get_user_threads(Path::from(user_id), Query(Some(10)))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Vec<Message> = parse_response_body(response).await;
        assert_eq!(body.len(), 2);
        assert_eq!(body[0].encrypted_content, "thread 1");
        assert_eq!(body[1].encrypted_content, "thread 2");
    }
}
