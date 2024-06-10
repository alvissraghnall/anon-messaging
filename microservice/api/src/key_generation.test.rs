use super::*;
use actix_web::test;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

// Helper function to setup test database
async fn setup_test_db() -> SqlitePool {
    let db_url = ":memory:";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Failed to create test database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

// Helper function to create test app
fn create_test_app(
    pool: web::Data<SqlitePool>,
) -> impl actix_web::dev::Service<actix_http::Request, Response = actix_web::dev::ServiceResponse> {
    test::init_service(
        App::new()
            .app_data(pool)
            .service(web::resource("/generate-keys").route(web::post().to(generate_keys))),
    )
}

#[actix_web::test]
async fn test_successful_key_generation() {
    // Setup
    env::set_var("SERVER_KEY", "test_server_key_12345678901234567890");
    let pool = setup_test_db().await;
    let app = create_test_app(web::Data::new(pool.clone())).await;

    // Create test request
    let request = KeyGenerationRequest {
        custom_user_id: None,
        keyphrase: "TestPass123!".to_string(),
    };

    // Make request
    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request)
        .send_request(&app)
        .await;

    // Assert response
    assert_eq!(resp.status(), 200);
    
    let response: KeyGenerationResponse = test::read_body_json(resp).await;
    assert!(!response.user_id.is_empty());
    assert!(!response.encrypted_private_key.is_empty());
    assert!(!response.encryption_nonce.is_empty());
    assert!(!response.encryption_salt.is_empty());

    // Verify database entry
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE user_id = ?",
        response.user_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(user.user_id, response.user_id);
    assert!(!user.public_key_hash.is_empty());
}

#[actix_web::test]
async fn test_custom_user_id() {
    env::set_var("SERVER_KEY", "test_server_key_12345678901234567890");
    let pool = setup_test_db().await;
    let app = create_test_app(web::Data::new(pool)).await;

    let custom_id = "test_user_123";
    let request = KeyGenerationRequest {
        custom_user_id: Some(custom_id.to_string()),
        keyphrase: "TestPass123!".to_string(),
    };

    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 200);
    let response: KeyGenerationResponse = test::read_body_json(resp).await;
    assert_eq!(response.user_id, custom_id);
}

#[actix_web::test]
async fn test_invalid_custom_user_id() {
    let pool = setup_test_db().await;
    let app = create_test_app(web::Data::new(pool)).await;

    // Test too long user_id
    let request = KeyGenerationRequest {
        custom_user_id: Some("a".repeat(21)),
        keyphrase: "TestPass123!".to_string(),
    };

    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 400);

    // Test invalid characters
    let request = KeyGenerationRequest {
        custom_user_id: Some("invalid@user#id".to_string()),
        keyphrase: "TestPass123!".to_string(),
    };

    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_invalid_keyphrase() {
    let pool = setup_test_db().await;
    let app = create_test_app(web::Data::new(pool)).await;

    // Test too short
    let request = KeyGenerationRequest {
        custom_user_id: None,
        keyphrase: "Short1".to_string(),
    };

    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 400);

    // Test missing uppercase
    let request = KeyGenerationRequest {
        custom_user_id: None,
        keyphrase: "lowercase123!".to_string(),
    };

    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 400);

    // Test missing lowercase
    let request = KeyGenerationRequest {
        custom_user_id: None,
        keyphrase: "UPPERCASE123!".to_string(),
    };

    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 400);

    // Test missing number
    let request = KeyGenerationRequest {
        custom_user_id: None,
        keyphrase: "NoNumbersHere!".to_string(),
    };

    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_duplicate_user_id() {
    env::set_var("SERVER_KEY", "test_server_key_12345678901234567890");
    let pool = setup_test_db().await;
    let app = create_test_app(web::Data::new(pool)).await;

    let request = KeyGenerationRequest {
        custom_user_id: Some("duplicate_user".to_string()),
        keyphrase: "TestPass123!".to_string(),
    };

    // First request should succeed
    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request.clone())
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 200);

    // Second request with same user_id should fail
    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 500);
}

#[test]
fn test_validate_user_id() {
    // Valid cases
    assert!(validate_user_id("valid_user_123").is_ok());
    assert!(validate_user_id("a").is_ok());
    assert!(validate_user_id("user_123").is_ok());

    // Invalid cases
    assert!(validate_user_id(&"a".repeat(21)).is_err());
    assert!(validate_user_id("invalid@user").is_err());
    assert!(validate_user_id("invalid space").is_err());
    assert!(validate_user_id("invalid#char").is_err());
}

#[test]
fn test_validate_keyphrase() {
    // Valid cases
    assert!(validate_keyphrase("ValidPass123!").is_ok());
    assert!(validate_keyphrase("AnotherValid123").is_ok());

    // Invalid cases
    assert!(validate_keyphrase("short1").is_err());
    assert!(validate_keyphrase("nouppercase123!").is_err());
    assert!(validate_keyphrase("NOLOWERCASE123!").is_err());
    assert!(validate_keyphrase("NoNumbersHere!").is_err());
}

