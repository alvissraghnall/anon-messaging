use super::*;
use actix_web::{test, App};
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

async fn setup_test_db() -> SqlitePool {
    let db_url = ":memory:";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Failed to create test database pool");

    sqlx::migrate!("../db/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

async fn create_test_app(pool: SqlitePool) -> impl actix_web::dev::Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error> {
    test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .service(web::resource("/generate-keys").route(web::post().to(generate_keys)))
    ).await
}

fn setup_test_env() {
    let test_key = "13c121c6e84dca7d31e852ad10148324".to_string();
    env::set_var("SERVER_KEY", test_key);
}

fn cleanup_test_env() {
    env::remove_var("SERVER_KEY");
}


#[actix_web::test]
async fn test_successful_key_generation() {
	setup_test_env();
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone()).await;

    let request = KeyGenerationRequest {
        custom_user_id: None,
        keyphrase: "TestPass123!".to_string(),
    };

    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 200);
    
    let response: KeyGenerationResponse = test::read_body_json(resp).await;
    assert!(!response.user_id.is_empty());
    assert!(!response.encrypted_private_key.is_empty());
    assert!(!response.encryption_nonce.is_empty());
    assert!(!response.encryption_salt.is_empty());

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
	setup_test_env();
    let pool = setup_test_db().await;
    let app = create_test_app(pool).await;

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
	setup_test_env();
    let pool = setup_test_db().await;
    let app = create_test_app(pool).await;

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
	setup_test_env();
    let pool = setup_test_db().await;
    let app = create_test_app(pool).await;

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
}

#[actix_web::test]
async fn test_duplicate_user_id() {
    setup_test_env();
	let pool = create_db_pool().await.expect("Failed to create database pool");
    let app = create_test_app(pool.clone()).await;
    
    let request = KeyGenerationRequest {
        custom_user_id: Some("duplicate_user".to_string()),
        keyphrase: "TestPass123!".to_string(),
    };

    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request.clone())
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 200);
    
    let first_response: serde_json::Value = test::read_body_json(resp).await;
    let first_user_id = first_response["user_id"].as_str().unwrap();
    
    let resp = test::TestRequest::post()
        .uri("/generate-keys")
        .set_json(&request)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 200);
    
    let second_response: serde_json::Value = test::read_body_json(resp).await;
    let second_user_id = second_response["user_id"].as_str().unwrap();
    
	println!("{} {}", first_user_id, second_user_id);
	println!("{:?}", first_response);
	println!("{:?}", second_response);
    assert_ne!(first_user_id, second_user_id);

	let query = "SELECT COUNT(*) FROM users WHERE user_id = ?";
    let count: (i64,) = sqlx::query_as(query)
		.bind("duplicate_user")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count.0, 1, "Expected one distinct user record in the database");
    

    let query = "SELECT COUNT(*) FROM users WHERE user_id IN (?, ?)";    
    let count: (i64,) = sqlx::query_as(query)
        .bind(first_user_id)
        .bind(second_user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count.0, 2);
}

#[test]
async fn test_validate_user_id() {
    assert!(validate_user_id("valid_user_123").is_ok());
    assert!(validate_user_id("a").is_ok());
    assert!(validate_user_id("user_123").is_ok());

    assert!(validate_user_id(&"a".repeat(21)).is_err());
    assert!(validate_user_id("invalid@user").is_err());
    assert!(validate_user_id("invalid space").is_err());
    assert!(validate_user_id("invalid#char").is_err());
}

#[test]
async fn test_validate_keyphrase() {
    assert!(validate_keyphrase("ValidPass123!").is_ok());
    assert!(validate_keyphrase("AnotherValid123").is_ok());

    assert!(validate_keyphrase("short1").is_err());
    assert!(validate_keyphrase("nouppercase123!").is_err());
    assert!(validate_keyphrase("NOLOWERCASE123!").is_err());
    assert!(validate_keyphrase("NoNumbersHere!").is_err());
}

