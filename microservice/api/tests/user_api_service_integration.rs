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
        public_key: "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAv28LFRWY1W1Q-LlMgdbE4c31KDXxScRtL28PpGl8TAkbx3LqLC3BwZkqO0bcwM5Bki1f4jqTkBL9apvFJolxXShu8Y6H7XWIq9rAtX1FOmYEOFV9da6P95xPzkhRdQo5IVnnWRpOSd2Ek5crwB1C6lTQRREbecp6-0FLgruF0R6yGnhiJsN30oncPDX6WlMjfPvfIBXGzOgbg3kHwbWhAXp5LMvJN5m63-RRXUbxyRTb7Gk-V3iHAAPQSmW93-udbp0zrOz4H8QDot_Pzdqrx3WpcInSDJ4a12G7fDdhOrHG9Rozq19WCVZ1QTqyZfgIOTXkeMIy_pjfSyFUfd462wIDAQAB".to_string(),
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
        public_key: "TUlJQklqQU5CZ2txaGtpRzl3MEJBUUVGQUFPQ0FROEFNSUlCQ2dLQ0FRRUF2MjhMRlJXWTFXMVEtTGxNZ2RiRTRjMzFLRFh4U2NSdEwyOFBwR2w4VEFrYngzTHFMQzNCd1prcU8wYmN3TTVCa2kxZjRqcVRrQkw5YXB2RkpvbHhYU2h1OFk2SDdYV0lxOXJBdFgxRk9tWUVPRlY5ZGE2UDk1eFB6a2hSZFFvNUlWbm5XUnBPU2QyRWs1Y3J3QjFDNmxUUVJSRWJlY3A2LTBGTGdydUYwUjZ5R25oaUpzTjMwb25jUERYNldsTWpmUHZmSUJYR3pPZ2JnM2tId2JXaEFYcDVMTXZKTjVtNjMtUlJYVWJ4eVJUYjdHay1WM2lIQUFQUVNtVzkzLXVkYnAwenJPejRIOFFEb3RfUHpkcXJ4M1dwY0luU0RKNGExMkc3ZkRkaE9ySEc5Um96cTE5V0NWWjFRVHF5WmZnSU9UWGtlTUl5X3BqZlN5RlVmZDQ2MndJREFRQUI".to_string(),
        username: Some("duplicate".to_string()),
    };
    
    let req = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&first_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
	let status = resp.status().as_u16();
	println!("{:?}", resp.into_body());
    assert_eq!(status, 201);
    
    // Then try to register another with the same username
    let second_request = RegisterRequest {
        public_key: "TUlJQklqQU5CZ2txaGtpRzl3MEJBUUVGQUFPQ0FROEFNSUlCQ2dLQ0FRRUF2MjhMRlJXWTFXMVEtTGxNZ2RiRTRjMzFLRFh4U2NSdEwyOFBwR2w4VEFrYngzTHFMQzNCd1prcU8wYmN3TTVCa2kxZjRqcVRrQkw5YXB2RkpvbHhYU2h1OFk2SDdYV0lxOXJBdFgxRk9tWUVPRlY5ZGE2UDk1eFB6a2hSZFFvNUlWbm5XUnBPU2QyRWs1Y3J3QjFDNmxUUVJSRWJlY3A2LTBGTGdydUYwUjZ5R25oaUpzTjMwb25jUERYNldsTWpmUHZmSUJYR3pPZ2JnM2tId2JXaEFYcDVMTXZKTjVtNjMtUlJYVWJ4eVJUYjdHay1WM2lIQUFQUVNtVzkzLXVkYnAwenJPejRIOFFEb3RfUHpkcXJ4M1dwY0luU0RKNGExMkc3ZkRkaE9ySEc5Um96cTE5V0NWWjFRVHF5WmZnSU9UWGtlTUl5X3BqZlN5RlVmZDQ2MndJREFRQUI".to_string(),
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
        public_key: "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAv28LFRWY1W1Q-LlMgdbE4c31KDXxScRtL28PpGl8TAkbx3LqLC3BwZkqO0bcwM5Bki1f4jqTkBL9apvFJolxXShu8Y6H7XWIq9rAtX1FOmYEOFV9da6P95xPzkhRdQo5IVnnWRpOSd2Ek5crwB1C6lTQRREbecp6-0FLgruF0R6yGnhiJsN30oncPDX6WlMjfPvfIBXGzOgbg3kHwbWhAXp5LMvJN5m63-RRXUbxyRTb7Gk-V3iHAAPQSmW93-udbp0zrOz4H8QDot_Pzdqrx3WpcInSDJ4a12G7fDdhOrHG9Rozq19WCVZ1QTqyZfgIOTXkeMIy_pjfSyFUfd462wIDAQAB".to_string(),
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
                public_key: format!("MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAv28LFRWY1W1Q-LlMgdbE4c31KDXxScRtL28PpGl8TAkbx3LqLC3BwZkqO0bcwM5Bki1f4jqTkBL9apvFJolxXShu8Y6H7XWIq9rAtX1FOmYEOFV9da6P95xPzkhRdQo5IVnnWRpOSd2Ek5crwB1C6lTQRREbecp6-0FLgruF0R6yGnhiJsN30oncPDX6WlMjfPvfIBXGzOgbg3kHwbWhAXp5LMvJN5m63-RRXUbxyRTb7Gk-V3iHAAPQSmW93-udbp0zrOz4H8QDot_Pzdqrx3WpcInSDJ4a12G7fDdhOrHG9Rozq19WCVZ1QTqyZfgIOTXkeMIy_pjfSyFUfd4{}2wIDAQAB", i),
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

#[actix_web::test]
async fn test_register_user_conflict() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await;

    let user_service = UserService::new(pool.clone());
    let controller: Arc<dyn UserController> = Arc::new(UserControllerImpl::new(web::Data::new(user_service)));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(controller))
            .configure(configure_routes)
    ).await;

    // First registration attempt
    let register_request = RegisterRequest {
        public_key: "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAv28LFRWY1W1Q-LlMgdbE4c31KDXxScRtL28PpGl8TAkbx3LqLC3BwZkqO0bcwM5Bki1f4jqTkBL9apvFJolxXShu8Y6H7XWIq9rAtX1FOmYEOFV9da6P95xPzkhRdQo5IVnnWRpOSd2Ek5crwB1C6lTQRREbecp6-0FLgruF0R6yGnhiJsN30oncPDX6WlMjfPvfIBXGzOgbg3kHwbWhAXp5LMvJN5m63-RRXUbxyRTb7Gk-V3iHAAPQSmW93-udbp0zrOz4H8QDot_Pzdqrx3WpcInSDJ4a12G7fDdhOrHG9Rozq19WCVZ1QTqyZfgIOTXkeMIy_pjfSyFUfd462wIDAQAB".to_string(),
        username: Some("conflictuser".to_string()),
    };

    let register_req1 = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&register_request)
        .to_request();

    let register_resp1 = test::call_service(&app, register_req1).await;
    assert_eq!(register_resp1.status().as_u16(), 201);

    // Second registration attempt with same public key and username
    let register_req2 = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&register_request)
        .to_request();

    let register_resp2 = test::call_service(&app, register_req2).await;

    println!("{:?}", register_resp2);
    assert_eq!(register_resp2.status().as_u16(), 409);

    let conflict_body: serde_json::Value = test::read_body_json(register_resp2).await;
    println!("{:?}", conflict_body);
}
