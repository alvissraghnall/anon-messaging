use actix_web::{http::StatusCode, test::{self}, web, App};
use actix_web::dev::{Service, ServiceResponse};
use actix_web::test::TestRequest;
use actix_web::http::header::ContentType;
use actix_http::Request;
use sqlx::types::chrono::Utc;
use db::{
    models::{Message, User}, 
    uuid::Uuid, 
    SqlitePool
};
use service::message::{
    repository::MessageRepository,
    service::MessageService,
};
use shared::{
    errors::AppError,
    models::{CreateMessageRequest, CreateMessageResponse},
};
use std::sync::Arc;
use api::message::{
    configure_routes, 
    MessageController, 
    MessageControllerImpl
};

mod common;
use common::{create_test_connection_pool, create_test_users, create_test_user_with_id};

async fn setup_test_app() -> (impl Service<Request, Response = ServiceResponse, Error = actix_web::Error>, SqlitePool, Uuid, Uuid) {
    let pool = create_test_connection_pool().await.unwrap();
    
    let users = create_test_users(&pool, 2).await.unwrap();
    let user1_id = users[0];
    let user2_id = users[1];
    
    let message_service = web::Data::new(MessageService::new(pool.clone()));
    
    let message_controller = web::Data::new(
        Arc::new(MessageControllerImpl::new(message_service)) as Arc<dyn MessageController>
    );
    
    let app = test::init_service(
        App::new()
            .app_data(message_controller.clone())
            .configure(configure_routes)
    ).await;
    
    (app, pool, user1_id, user2_id)
}

#[actix_web::test]
async fn test_create_message() {
    let (app, pool, sender_id, recipient_id) = setup_test_app().await;
    let enc_content = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAxpaKTGz1LlgVihe0dGlE";
    let sig = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAxpaKTGz1LlgVihe0dGlE";
    
    let request = CreateMessageRequest {
        sender_id,
        recipient_id,
        encrypted_content: enc_content.to_string(),
        signature: Some(sig.to_string()),
        parent_id: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/messages")
        .set_json(&request)
        .to_request();

    println!("{:?}", request);
    let response: CreateMessageResponse = test::call_and_read_body_json(&app, req).await;

    println!("{:?}", response.get_message_id());
    assert!(response.get_message_id() > 0);
    
    let message = pool.get_message_by_id(response.get_message_id()).await.unwrap();
    assert!(message.is_some());
    
    let message = message.unwrap();
    assert_eq!(message.sender_id, sender_id);
    assert_eq!(message.recipient_id, recipient_id);
    assert_eq!(message.encrypted_content, enc_content);
    assert_eq!(message.signature, Some(sig.to_string()));
    assert!(message.parent_id.is_none());
}

#[actix_web::test]
async fn test_get_message() {
    let (app, pool, sender_id, recipient_id) = setup_test_app().await;

    let enc_content = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAxpaKTGz1LlgVihe0dGlE";
    let sig = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAxpaKTGz1LlgVihe0dGlE";
    
    let message_id = pool.insert_message(
        sender_id,
        recipient_id,
        enc_content,
        Some(sig.to_string()),
        None
    ).await.unwrap().unwrap();
    
    let req = test::TestRequest::get()
        .uri(&format!("/api/messages/{}", message_id))
        .to_request();
    
    let message: Message = test::call_and_read_body_json(&app, req).await;
    
    assert_eq!(message.id, message_id);
    assert_eq!(message.sender_id, sender_id);
    assert_eq!(message.recipient_id, recipient_id);
    assert_eq!(message.encrypted_content, enc_content);
    assert_eq!(message.signature, Some(sig.to_string()));
    assert!(message.parent_id.is_none());
}

#[actix_web::test]
async fn test_get_message_not_found() {
    let (app, _pool, _sender_id, _recipient_id) = setup_test_app().await;
    
    let req = test::TestRequest::get()
        .uri("/api/messages/999999")
        .to_request();
    
    let response = test::call_service(&app, req).await;
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_get_conversation() {
    let (app, pool, user1_id, user2_id) = setup_test_app().await;
    
    // Create a few messages between the two users
    pool.insert_message(
        user1_id,
        user2_id,
        "Message 1 from user1 to user2",
        None,
        None
    ).await.unwrap();
    
    pool.insert_message(
        user2_id,
        user1_id,
        "Reply from user2 to user1",
        None,
        None
    ).await.unwrap();
    
    pool.insert_message(
        user1_id,
        user2_id,
        "Second message from user1 to user2",
        None,
        None
    ).await.unwrap();
    
    // Create a message between user1 and another user (should not be in the conversation)
    let user3_id = Uuid::now_v7();
    let _ = create_test_user_with_id(&pool, user3_id.clone()).await.unwrap();

    println!("{}", 54);    
    pool.insert_message(
        user1_id,
        user3_id,
        "Message to user3",
        None,
        None
    ).await.unwrap();
    println!("{}", 54);
    
    let req = test::TestRequest::get()
        .uri(&format!("/api/messages/conversations/{}/{}", user1_id, user2_id))
        .to_request();
    
    let messages: Vec<Message> = test::call_and_read_body_json(&app, req).await;
    println!("{:?}", messages);
    
    assert_eq!(messages.len(), 3);
    
    // Verify all messages are between user1 and user2
    for message in &messages {
        assert!(
            (message.sender_id == user1_id && message.recipient_id == user2_id) ||
            (message.sender_id == user2_id && message.recipient_id == user1_id)
        );
    }
}

#[actix_web::test]
async fn test_thread_replies() {
    let (app, pool, user1_id, user2_id) = setup_test_app().await;
    
    let parent_id = pool.insert_message(
        user1_id,
        user2_id,
        "Parent message",
        None,
        None
    ).await.unwrap().unwrap();
    
    // Create some replies to the parent message
    pool.insert_message(
        user2_id,
        user1_id,
        "Reply 1",
        None,
        Some(parent_id)
    ).await.unwrap();
    
    pool.insert_message(
        user1_id,
        user2_id,
        "Reply 2",
        None,
        Some(parent_id)
    ).await.unwrap();
    
    pool.insert_message(
        user2_id,
        user1_id,
        "Reply 3",
        None,
        Some(parent_id)
    ).await.unwrap();
    
    // Create a message that's not a reply to this thread
    pool.insert_message(
        user1_id,
        user2_id,
        "Not a reply to this thread",
        None,
        None
    ).await.unwrap();
    
    // Get the thread replies
    let req = test::TestRequest::get()
        .uri(&format!("/api/messages/threads/{}/replies", parent_id))
        .to_request();
    
    let replies: Vec<Message> = test::call_and_read_body_json(&app, req).await;
    
    assert_eq!(replies.len(), 3);
    
    // Verify all messages are replies to the parent
    for reply in &replies {
        assert_eq!(reply.parent_id, Some(parent_id));
    }
}

#[actix_web::test]
async fn test_complete_thread() {
    let (app, pool, user1_id, user2_id) = setup_test_app().await;
    
    // Create a parent message
    let parent_id = pool.insert_message(
        user1_id,
        user2_id,
        "Thread root message",
        None,
        None
    ).await.unwrap().unwrap();
    
    // Create some replies to the parent message
    let reply1_id = pool.insert_message(
        user2_id,
        user1_id,
        "Reply 1",
        None,
        Some(parent_id)
    ).await.unwrap().unwrap();
    
    pool.insert_message(
        user1_id,
        user2_id,
        "Reply 2",
        None,
        Some(parent_id)
    ).await.unwrap();
    
    // Create a nested reply (reply to a reply)
    pool.insert_message(
        user2_id,
        user1_id,
        "Nested reply",
        None,
        Some(reply1_id)
    ).await.unwrap();
    
    // Get the complete thread
    let req = test::TestRequest::get()
        .uri(&format!("/api/messages/threads/{}", parent_id))
        .to_request();
    
    let thread: Vec<Message> = test::call_and_read_body_json(&app, req).await;
    
    assert_eq!(thread.len(), 4);
    
    // The first message should be the thread root
    assert_eq!(thread[0].id, parent_id);
    assert!(thread[0].parent_id.is_none());
}

#[actix_web::test]
async fn test_user_threads() {
    let (app, pool, user1_id, user2_id) = setup_test_app().await;
    
    let user3_id = Uuid::now_v7();   
    let user3 = create_test_user_with_id(
        &pool, 
        user3_id
    ).await.unwrap();
    
    // Create some thread root messages by user1
    let thread1_id = pool.insert_message(
        user1_id,
        user2_id,
        "Thread 1 root",
        None,
        None
    ).await.unwrap().unwrap();
    
    let thread2_id = pool.insert_message(
        user1_id,
        user3_id,
        "Thread 2 root",
        None,
        None
    ).await.unwrap().unwrap();
    
    // Create some replies
    pool.insert_message(
        user2_id,
        user1_id,
        "Reply to thread 1",
        None,
        Some(thread1_id)
    ).await.unwrap();
    
    pool.insert_message(
        user3_id,
        user1_id,
        "Reply to thread 2",
        None,
        Some(thread2_id)
    ).await.unwrap();
    
    // Create a thread started by another user
    let other_thread_id = pool.insert_message(
        user2_id,
        user3_id,
        "Thread by user2",
        None,
        None
    ).await.unwrap().unwrap();
    
    // User1 replies to this thread
    pool.insert_message(
        user1_id,
        user2_id,
        "User1 reply to user2's thread",
        None,
        Some(other_thread_id)
    ).await.unwrap();
    
    // Get user1's threads
    let req = test::TestRequest::get()
        .uri(&format!("/api/messages/users/{}/threads", user1_id))
        .to_request();
    
    let threads: Vec<Message> = test::call_and_read_body_json(&app, req).await;
    
    assert_eq!(threads.len(), 2);
    
    // Verify all threads are started by user1
    for thread in &threads {
        assert_eq!(thread.sender_id, user1_id);
        assert!(thread.parent_id.is_none());
    }
    
    // The thread IDs should match our created threads
    let thread_ids: Vec<i64> = threads.iter().map(|t| t.id).collect();
    assert!(thread_ids.contains(&thread1_id));
    assert!(thread_ids.contains(&thread2_id));
}

#[actix_web::test]
async fn test_create_message_validation() {
    let (app, _pool, sender_id, _recipient_id) = setup_test_app().await;
    
    let request = CreateMessageRequest {
        sender_id,
        recipient_id: Uuid::nil(),
        encrypted_content: "".to_string(), 
        signature: None,
        parent_id: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/messages")
        .set_json(&request)
        .to_request();
    
    let response = test::call_service(&app, req).await;
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_get_conversation_with_limit() {
    let (app, pool, user1_id, user2_id) = setup_test_app().await;
    
    // Create several messages between the two users
    for i in 1..=5 {
        pool.insert_message(
            user1_id,
            user2_id,
            &format!("Message {}", i),
            None,
            None
        ).await.unwrap();
    }
    
    let req = test::TestRequest::get()
        .uri(&format!("/api/messages/conversations/{}/{}?limit=3", user1_id, user2_id))
        .to_request();
    
    let messages: Vec<Message> = test::call_and_read_body_json(&app, req).await;
    
    // Verify we got the right number of messages
    assert_eq!(messages.len(), 3);
}

#[actix_web::test]
async fn test_thread_replies_with_pagination() {
    let (app, pool, user1_id, user2_id) = setup_test_app().await;
    
    // Create a parent message
    let parent_id = pool.insert_message(
        user1_id,
        user2_id,
        "Parent message",
        None,
        None
    ).await.unwrap().unwrap();
    
    // Create several replies
    for i in 1..=5 {
        pool.insert_message(
            if i % 2 == 0 { user1_id } else { user2_id },
            if i % 2 == 0 { user2_id } else { user1_id },
            &format!("Reply {}", i),
            None,
            Some(parent_id)
        ).await.unwrap();
    }
    
    // Get the replies with limit=2 and offset=2
    let req = test::TestRequest::get()
        .uri(&format!("/api/messages/threads/{}/replies?limit=2&offset=2", parent_id))
        .to_request();
    
    let replies: Vec<Message> = test::call_and_read_body_json(&app, req).await;
    
    assert_eq!(replies.len(), 2);
    
    for reply in &replies {
        assert_eq!(reply.parent_id, Some(parent_id));
    }
    
    // Verify we got the right replies (3rd and 4th)
    assert_eq!(replies[0].encrypted_content, "Reply 3");
    assert_eq!(replies[1].encrypted_content, "Reply 4");
}
