use actix_web::{test, App, web::{self, Data}};
use db::{SqlitePool, models::User};
use service::user::UserService;
use shared::models::{RegisterRequest, RegisterResponse, UpdateUserRequest};
use std::sync::Arc;
use db::uuid::Uuid;

use api::{user::{configure_routes, UserController, UserControllerImpl}};

#[actix_web::test]
async fn test_register_user() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;
        
    let service = Data::new(UserService::new(pool.clone()));
    let controller = Arc::new(UserControllerImpl::new(service)) as Arc<dyn UserController>;
    
    let app = test::init_service(
        App::new()
            .app_data(Data::new(controller.clone()))
            .configure(configure_routes)
    ).await;
    
    let request = RegisterRequest {
        public_key: "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAxpaKTGz1LlgVihe0dGlE".to_string(),
        username: Some("testuser".to_string()),
    };
    
    let req = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status().as_u16(), 201);
    
    let response: RegisterResponse = test::read_body_json(resp).await;
    assert_eq!(response.username, "testuser");
    assert!(response.user_id != Uuid::nil());
}

#[actix_web::test]
async fn test_register_user_without_username() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;
    
    let service = Data::new(UserService::new(pool.clone()));
    let controller = Arc::new(UserControllerImpl::new(service)) as Arc<dyn UserController>;
    
    let app = test::init_service(
        App::new()
            .app_data(Data::new(controller.clone()))
            .configure(configure_routes)
    ).await;
    
    let request = RegisterRequest {
        public_key: "TUlJQklqQU5CZ2txaGtpRzl3MEJBUUVGQUFPQ0FROEFNSUlCQ2dLQ0FRRUF2MjhMRlJXWTFXMVEtTGxNZ2RiRTRjMzFLRFh4U2NSdEwyOFBwR2w4VEFrYngzTHFMQzNCd1prcU8wYmN3TTVCa2kxZjRqcVRrQkw5YXB2RkpvbHhYU2h1OFk2SDdYV0lxOXJBdFgxRk9tWUVPRlY5ZGE2UDk1eFB6a2hSZFFvNUlWbm5XUnBPU2QyRWs1Y3J3QjFDNmxUUVJSRWJlY3A2LTBGTGdydUYwUjZ5R25oaUpzTjMwb25jUERYNldsTWpmUHZmSUJYR3pPZ2JnM2tId2JXaEFYcDVMTXZKTjVtNjMtUlJYVWJ4eVJUYjdHay1WM2lIQUFQUVNtVzkzLXVkYnAwenJPejRIOFFEb3RfUHpkcXJ4M1dwY0luU0RKNGExMkc3ZkRkaE9ySEc5Um96cTE5V0NWWjFRVHF5WmZnSU9UWGtlTUl5X3BqZlN5RlVmZDQ2MndJREFRQUI".to_string(),
        username: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status().as_u16(), 201);
    
    let response: RegisterResponse = test::read_body_json(resp).await;
    assert!(!response.username.is_empty());
    assert!(response.user_id != Uuid::nil());
}

#[actix_web::test]
async fn test_get_user_by_id() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;
    
    let service = Data::new(UserService::new(pool.clone()));
    let controller = Arc::new(UserControllerImpl::new(service.clone())) as Arc<dyn UserController>;
    
    let app = test::init_service(
        App::new()
            .app_data(Data::new(controller.clone()))
            .configure(configure_routes)
    ).await;
    
    let request = RegisterRequest {
        public_key: "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAw5VO".to_string(),
        username: Some("findme".to_string()),
    };
    
    let response = service.register_user(request).await.unwrap();
    let user_id = response.user_id;
    
    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{}", user_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status().as_u16(), 200);
    
    let user: User = test::read_body_json(resp).await;
    assert_eq!(user.username, "findme");
    assert_eq!(user.id, user_id);
}

#[actix_web::test]
async fn test_get_user_not_found() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;
    
    let service = Data::new(UserService::new(pool.clone()));
    let controller = Arc::new(UserControllerImpl::new(service)) as Arc<dyn UserController>;
    
    let app = test::init_service(
        App::new()
            .app_data(Data::new(controller.clone()))
            .configure(configure_routes)
    ).await;
    
    let random_uuid = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{}", random_uuid))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status().as_u16(), 404);
}

#[actix_web::test]
async fn test_get_users() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;
    
    let service = Data::new(UserService::new(pool.clone()));
    let controller = Arc::new(UserControllerImpl::new(service.clone())) as Arc<dyn UserController>;
    
    let app = test::init_service(
        App::new()
            .app_data(Data::new(controller.clone()))
            .configure(configure_routes)
    ).await;
    
    // Create multiple users
    for i in 1..=3 {
        let request = RegisterRequest {
            public_key: format!("VGhlIHN1biBzaGFsbCBzb29uIHNoaW{}l", i),
            username: Some(format!("user{}", i)),
        };
        service.register_user(request).await.unwrap();
    }
    
    let req = test::TestRequest::get()
        .uri("/api/users")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status().as_u16(), 200);
    
    let users: Vec<User> = test::read_body_json(resp).await;
    assert_eq!(users.len(), 3);
}

#[actix_web::test]
async fn test_get_users_with_limit() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;
    
    let service = Data::new(UserService::new(pool.clone()));
    let controller = Arc::new(UserControllerImpl::new(service.clone())) as Arc<dyn UserController>;
    
    let app = test::init_service(
        App::new()
            .app_data(Data::new(controller.clone()))
            .configure(configure_routes)
    ).await;
    
    for i in 1..=5 {
        let request = RegisterRequest {
            public_key: format!("MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCQEA{}", i),
            username: Some(format!("limited{}", i)),
        };
        service.register_user(request).await.unwrap();
    }
    
    let req = test::TestRequest::get()
        .uri("/api/users?limit=2")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status().as_u16(), 200);
    
    let users: Vec<User> = test::read_body_json(resp).await;
    assert_eq!(users.len(), 2);
}

#[actix_web::test]
async fn test_update_user() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;
    
    let service = Data::new(UserService::new(pool.clone()));
    let controller = Arc::new(UserControllerImpl::new(service.clone())) as Arc<dyn UserController>;
    
    let app = test::init_service(
        App::new()
            .app_data(Data::new(controller.clone()))
            .configure(configure_routes)
    ).await;
    
    let request = RegisterRequest {
        public_key: "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA56po".to_string(),
        username: Some("beforeupdate".to_string()),
    };
    
    let response = service.register_user(request).await.unwrap();
    let user_id = response.user_id;
    
    let update_request = UpdateUserRequest {
        new_username: Some("afterupdate".to_string()),
        new_public_key: None,
    };
    
    let req = test::TestRequest::patch()
        .uri(&format!("/api/users/{}", user_id))
        .set_json(&update_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status().as_u16(), 200);
    
    // Verify the update by fetching the user
    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{}", user_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    let updated_user: User = test::read_body_json(resp).await;
    assert_eq!(updated_user.username, "afterupdate");
}

#[actix_web::test]
async fn test_update_user_public_key() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;
    
    let service = Data::new(UserService::new(pool.clone()));
    let controller = Arc::new(UserControllerImpl::new(service.clone())) as Arc<dyn UserController>;
    
    let app = test::init_service(
        App::new()
            .app_data(Data::new(controller.clone()))
            .configure(configure_routes)
    ).await;
    
    let old_public_key = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAnzpQ".to_string();
    let request = RegisterRequest {
        public_key: old_public_key.clone(),
        username: Some("keyupdateuser".to_string()),
    };
    
    let response = service.register_user(request).await.unwrap();
    let user_id = response.user_id;
    
    let initial_user = service.get_user_by_id(user_id).await.unwrap();
    
    // Execute update request with new public key
    let new_public_key = "TUlJQklqQU5CZ2txaGtpRzl3MEJBUUVGQUFPQ0FROEFNSUlCQ2dLQ0FRRUF2MjhMRlJXWTFXMVEtTGxNZ2RiRTRjMzFLRFh4U2NSdEwyOFBwR2w4VEFrYngzTHFMQzNCd1prcU8wYmN3TTVCa2kxZjRqcVRrQkw5YXB2RkpvbHhYU2h1OFk2SDdYV0lxOXJBdFgxRk9tWUVPRlY5ZGE2UDk1eFB6a2hSZFFvNUlWbm5XUnBPU2QyRWs1Y3J3QjFDNmxUUVJSRWJlY3A2LTBGTGdydUYwUjZ5R25oaUpzTjMwb25jUERYNldsTWpmUHZmSUJYR3pPZ2JnM2tId2JXaEFYcDVMTXZKTjVtNjMtUlJYVWJ4eVJUYjdHay1WM2lIQUFQUVNtVzkzLXVkYnAwenJPejRIOFFEb3RfUHpkcXJ4M1dwY0luU0RKNGExMkc3ZkRkaE9ySEc5Um96cTE5V0NWWjFRVHF5WmZnSU9UWGtlTUl5X3BqZlN5RlVmZDQ2MndJREFRQUI".to_string();
    let update_request = UpdateUserRequest {
        new_username: None,
        new_public_key: Some(new_public_key.clone()),
    };
    
    let req = test::TestRequest::patch()
        .uri(&format!("/api/users/{}", user_id))
        .set_json(&update_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status().as_u16(), 200);
    
    // Verify the public key hash was updated
    let updated_hash = service.fetch_public_key_hash(user_id).await.unwrap();
    println!("{:?} {}", updated_hash, initial_user.public_key_hash.to_string());
    assert_ne!(updated_hash, initial_user.public_key_hash.to_string());
    
    // Try to get user by new public key
    let updated_user = service.get_user_by_public_key(&new_public_key).await.unwrap();
    assert_eq!(updated_user.id, user_id);
}

#[actix_web::test]
async fn test_update_user_not_found() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;
    
    let service = Data::new(UserService::new(pool.clone()));
    let controller = Arc::new(UserControllerImpl::new(service)) as Arc<dyn UserController>;
    
    let app = test::init_service(
        App::new()
            .app_data(Data::new(controller.clone()))
            .configure(configure_routes)
    ).await;
    
    // Execute update request with non-existent UUID
    let random_uuid = Uuid::new_v4();
    let update_request = UpdateUserRequest {
        new_username: Some("wontwork".to_string()),
        new_public_key: None,
    };
    
    let req = test::TestRequest::patch()
        .uri(&format!("/api/users/{}", random_uuid))
        .set_json(&update_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status().as_u16(), 404);
}
