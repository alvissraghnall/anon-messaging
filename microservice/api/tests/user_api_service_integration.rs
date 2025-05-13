use actix_web::{test, web, App};
use db::{SqlitePool, models::User};
use shared::models::{RegisterRequest, UpdateUserRequest};
use std::sync::Arc;
use db::uuid::Uuid;
use api::{user::{configure_routes, UserController, UserControllerImpl}};
use service::{user::UserService};
use futures::future::join_all;
use sqlx::migrate::Migrator;

#[actix_web::test]
async fn test_full_user_lifecycle() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;
    
    let user_service = UserService::new(pool.clone());
    let controller: Arc<dyn UserController> = Arc::new(UserControllerImpl::new(web::Data::new(user_service)));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(controller))
            .configure(configure_routes)
    ).await;
    
    // Step 1: Register a new user
    let register_request = RegisterRequest {
        public_key: "VGhlIHN1biBzaGFsbCBzb29uIHNoaW5l".to_string(),
        username: Some("lifecycleuser".to_string()),
    };
    
    let register_req = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&register_request)
        .to_request();
    
    let register_resp = test::call_service(&app, register_req).await;
    println!("{:?}", register_resp);

    assert_eq!(register_resp.status().as_u16(), 201);
    
    let register_body: serde_json::Value = test::read_body_json(register_resp).await;
    println!("{:?}", register_body);
    let user_id = register_body["user_id"].as_str().unwrap();
    let user_id = Uuid::parse_str(user_id).unwrap();
        
    // Step 2: Get the user by ID
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/users/{}", user_id))
        .to_request();
    
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status().as_u16(), 200);
    
    let user: User = test::read_body_json(get_resp).await;
    assert_eq!(user.username, "lifecycleuser");
    
    // Step 3: Update the user
    let update_request = UpdateUserRequest {
        new_username: Some("updateduser".to_string()),
        new_public_key: None,
    };
    
    let update_req = test::TestRequest::patch()
        .uri(&format!("/api/users/{}", user_id))
        .set_json(&update_request)
        .to_request();
    
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status().as_u16(), 200);
    
    // Step 4: Verify the update
    let verify_req = test::TestRequest::get()
        .uri(&format!("/api/users/{}", user_id))
        .to_request();
    
    let verify_resp = test::call_service(&app, verify_req).await;
    let updated_user: User = test::read_body_json(verify_resp).await;
    assert_eq!(updated_user.username, "updateduser");
    
    // Step 5: List all users and ensure our user is there
    let list_req = test::TestRequest::get()
        .uri("/api/users")
        .to_request();
    
    let list_resp = test::call_service(&app, list_req).await;
    println!("{:?}", list_resp);
    assert_eq!(list_resp.status().as_u16(), 200);
    
    let users: Vec<User> = test::read_body_json(list_resp).await;
    println!("{:?}", users);
    assert!(users.iter().any(|u| u.id == user_id));
}

#[actix_web::test]
async fn test_edge_cases() {
    
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;

    let user_service = UserService::new(pool.clone());
    let controller: Arc<dyn UserController> = Arc::new(UserControllerImpl::new(web::Data::new(user_service)));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(controller))
            .configure(configure_routes)
    ).await;
    
    // Case 1: Try to register with an invalid public key
    let invalid_request = RegisterRequest {
        public_key: "invalid-key".to_string(),
        username: Some("invalidkeyuser".to_string()),
    };
    
    let req = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&invalid_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400);
    
    // Case 2: Try to get a non-existent user
    let random_uuid = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{}", random_uuid))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404);
    
    // Case 3: Register with duplicate username
    // First register a valid user
    let first_request = RegisterRequest {
        public_key: "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAzp7h".to_string(),
        username: Some("duplicate".to_string()),
    };
    
    let req = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&first_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 201);
    
    // Then try to register another with the same username
    let second_request = RegisterRequest {
        public_key: "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAkr6j".to_string(),
        username: Some("duplicate".to_string()),
    };
    
    let req = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&second_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().as_u16() >= 400);
    
    // Case 4: Update a user with an invalid public key
    // First register a valid user
    let register_request = RegisterRequest {
        public_key: "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAtg7h".to_string(),
        username: Some("updateinvalid".to_string()),
    };
    
    let req = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&register_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 201);
    
    let register_body: serde_json::Value = test::read_body_json(resp).await;
    let user_id = register_body["user_id"].as_str().unwrap();
    let user_id = Uuid::parse_str(user_id).unwrap();
    
    // Try to update with invalid public key
    let update_request = UpdateUserRequest {
        new_username: None,
        new_public_key: Some("invalid-key".to_string()),
    };
    
    let req = test::TestRequest::patch()
        .uri(&format!("/api/users/{}", user_id))
        .set_json(&update_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400);
}

#[actix_web::test]
async fn test_concurrent_operations() {
    
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;

    let user_service = UserService::new(pool.clone());
    let controller: Arc<dyn UserController> = Arc::new(UserControllerImpl::new(web::Data::new(user_service)));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(controller))
            .configure(configure_routes)
    ).await;
    
    // Create 5 users concurrently    
    let mut futures = vec![];

    for i in 1..=5 {
        let user_service = UserService::new(pool.clone());
        let controller: Arc<dyn UserController> = Arc::new(UserControllerImpl::new(web::Data::new(user_service)));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(controller))
                .configure(configure_routes)
        ).await;

        let fut = async move {
            let register_request = RegisterRequest {
                public_key: format!("VGhlIHN1biBzaGFsbCBzb29uIHNoaW{}l", i),
                username: Some(format!("concurrent{}", i)),
            };

            let req = test::TestRequest::post()
                .uri("/api/users")
                .set_json(&register_request)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status().as_u16(), 201);

            let body: serde_json::Value = test::read_body_json(resp).await;
            let user_id_str = body["user_id"].as_str().unwrap();
            Uuid::parse_str(user_id_str).unwrap()
        };

        futures.push(fut);
    }

    let results = join_all(futures).await;

    let user_ids: Vec<Uuid> = results.into_iter().collect();
    
    // Verify all users were created
    let list_req = test::TestRequest::get()
        .uri("/api/users")
        .to_request();
    
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status().as_u16(), 200);
    
    let users: Vec<User> = test::read_body_json(list_resp).await;
    assert_eq!(users.len(), 5);
    
    // Prepare futures for concurrent updates
    let mut update_futures = Vec::new();

    for (i, user_id) in user_ids.iter().enumerate() {
        let user_service = UserService::new(pool.clone());
        let controller: Arc<dyn UserController> = Arc::new(UserControllerImpl::new(web::Data::new(user_service)));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(controller))
                .configure(configure_routes)
        ).await;
        let user_id = user_id.clone();

        let fut = async move {
            let update_request = UpdateUserRequest {
                new_username: Some(format!("updated{}", i + 1)),
                new_public_key: None,
            };

            let req = test::TestRequest::patch()
                .uri(&format!("/api/users/{}", user_id))
                .set_json(&update_request)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status().as_u16(), 200);
        };

        update_futures.push(fut);
    }

    // Run all updates concurrently
    join_all(update_futures).await;
    
    // Verify all updates were applied
    for (i, user_id) in user_ids.iter().enumerate() {
        let req = test::TestRequest::get()
            .uri(&format!("/api/users/{}", user_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 200);
        
        let user: User = test::read_body_json(resp).await;
        assert_eq!(user.username, format!("updated{}", i + 1));
    }
}
