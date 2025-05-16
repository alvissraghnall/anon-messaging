use actix_web::{test, web, App};
use db::{uuid::Uuid, SqlitePool};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use serde_json::json;
use service::token::{
    repository::TokenRepository,
    service::TokenService,
};
use shared::errors::AppError;
use std::time::{SystemTime, UNIX_EPOCH};
mod common;
use common::{create_test_connection_pool, create_test_users, create_test_user_with_id};
use api::token::{
    TokenControllerImpl, StoreTokenRequest, ValidateTokenRequest, 
    RevokeTokenRequest, ValidateTokenResponse, JwtConfig, JwtClaims, get_decoding_key
};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

static ENCODING_KEY: OnceLock<EncodingKey> = OnceLock::new();

fn get_encoding_key() -> &'static EncodingKey {
    ENCODING_KEY.get_or_init(|| {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../private_key.pem");

        let private_key = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to read private key file: {:?}", path));
        EncodingKey::from_rsa_pem(private_key.as_bytes())
            .expect("Failed to create encoding key")
    })
}

async fn get_test_db() -> SqlitePool {
    let database_url = "sqlite::memory:";
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to connect to test database");
    
    sqlx::migrate!("../db/migrations")
        .run(&pool)
        .await
        .expect("Failed to create schemas");
    
    pool
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}

fn future_timestamp(seconds_from_now: u64) -> i64 {
    current_timestamp() + seconds_from_now as i64
}

fn get_test_jwt_config() -> JwtConfig {
    let mut validation = jsonwebtoken::Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    
    JwtConfig {
        decoding_key: get_decoding_key().clone(),
        validation,
    }
}

fn create_test_jwt(user_id: &Uuid, expiry: i64, device_info: Option<String>) -> String {
    let claims = JwtClaims {
        sub: user_id.to_string(),
        exp: expiry,
        iat: current_timestamp(),
        device: device_info,
    };
    
    encode(
        &Header::new(Algorithm::RS256),
        &claims,
        &get_encoding_key()
    ).expect("Failed to create test JWT")
}

async fn get_test_app() -> (SqlitePool, impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
>) {
    let db = get_test_db().await;
    let jwt_config = get_test_jwt_config();
    let controller = TokenControllerImpl::new_with_config(db.clone(), jwt_config);
    
    (db.clone(), test::init_service(
        App::new()
            .app_data(web::Data::new(controller))
            .app_data(web::JsonConfig::default().limit(4096))
            .configure(TokenControllerImpl::configure(db))
            
    ).await)
}

#[actix_web::test]
async fn test_store_token_handler() {
    let (db, app) = get_test_app().await;
    let user = create_test_users(&db, 1).await.unwrap();
    let user_id = user[0];

    let expiry = future_timestamp(3600); // 1 hour from now
    let device_info = Some("Test Device".to_string());
    let jwt_token = create_test_jwt(&user_id, expiry, device_info.clone());
    println!("{:?}", jwt_token);
    
    let request_body = json!({
        "jwt_token": jwt_token,
        "device_info": device_info
    });
    
    let req = test::TestRequest::post()
        .uri("/api/tokens/store")
        .set_json(&request_body)
        .to_request();
    
    let resp = test::call_service(&app, req).await;

    let status = resp.status();

    println!("{:?}", resp.into_body());
    
    assert_eq!(status, 201, "Response status should be 201 Created");
}

#[actix_web::test]
async fn test_store_and_validate_token() {
    let (db, app) = get_test_app().await;
    let user = create_test_users(&db, 1).await.unwrap();
    let user_id = user[0];
    let expiry = future_timestamp(3600); // 1 hour from now
    let device_info = Some("Test Browser".to_string());
    let jwt_token = create_test_jwt(&user_id, expiry, device_info.clone());
    
    let store_req_body = json!({
        "jwt_token": jwt_token,
        "device_info": device_info
    });
    
    let store_req = test::TestRequest::post()
        .uri("/api/tokens/store")
        .set_json(&store_req_body)
        .to_request();
    
    let store_resp = test::call_service(&app, store_req).await;
    assert_eq!(store_resp.status(), 201, "Failed to store token");
    
    // Act - Validate the token
    let validate_req_body = json!({
        "jwt_token": jwt_token
    });
    
    let validate_req = test::TestRequest::post()
        .uri("/api/tokens/validate")
        .set_json(&validate_req_body)
        .to_request();
    
    let validate_resp = test::call_service(&app, validate_req).await;
    
    // Assert
    assert_eq!(validate_resp.status(), 200, "Response status should be 200 OK");
    
    let response_body: ValidateTokenResponse = test::read_body_json(validate_resp).await;
    assert!(response_body.valid, "Token should be valid");
}

#[actix_web::test]
async fn test_validate_non_existent_token() {
    let (db, app) = get_test_app().await;
    let user = create_test_users(&db, 1).await.unwrap();
    let user_id = user[0];
    let expiry = future_timestamp(3600);
    let jwt_token = create_test_jwt(&user_id, expiry, None);
    
    // Validate a token that hasn't been stored
    let validate_req_body = json!({
        "jwt_token": jwt_token
    });
    
    let validate_req = test::TestRequest::post()
        .uri("/api/tokens/validate")
        .set_json(&validate_req_body)
        .to_request();
    
    let validate_resp = test::call_service(&app, validate_req).await;
    
    assert_eq!(validate_resp.status(), 200, "Response status should be 200 OK");
    
    let response_body: ValidateTokenResponse = test::read_body_json(validate_resp).await;
    assert!(!response_body.valid, "Non-existent token should not be valid");
}

#[actix_web::test]
async fn test_store_validate_revoke_validate() {
    let (db, app) = get_test_app().await;
    let user = create_test_users(&db, 1).await.unwrap();
    let user_id = user[0];
    let expiry = future_timestamp(3600);
    let jwt_token = create_test_jwt(&user_id, expiry, None);
    
    // Step 1: Store the token
    let store_req_body = json!({
        "jwt_token": jwt_token,
        "device_info": null
    });
    
    let store_req = test::TestRequest::post()
        .uri("/api/tokens/store")
        .set_json(&store_req_body)
        .to_request();
    
    let store_resp = test::call_service(&app, store_req).await;
    assert_eq!(store_resp.status(), 201, "Failed to store token");
    
    // Step 2: Validate the token (should be valid)
    let validate_req_body = json!({
        "jwt_token": jwt_token
    });
    
    let validate_req = test::TestRequest::post()
        .uri("/api/tokens/validate")
        .set_json(&validate_req_body)
        .to_request();
    
    let validate_resp = test::call_service(&app, validate_req).await;
    assert_eq!(validate_resp.status(), 200);
    
    let response_body: ValidateTokenResponse = test::read_body_json(validate_resp).await;
    assert!(response_body.valid, "Token should be valid before revocation");
    
    // Step 3: Revoke the token
    let revoke_reason = "Test revocation reason";
    let revoke_req_body = json!({
        "jwt_token": jwt_token,
        "reason": revoke_reason
    });
    
    let revoke_req = test::TestRequest::post()
        .uri("/api/tokens/revoke")
        .set_json(&revoke_req_body)
        .to_request();
    
    let revoke_resp = test::call_service(&app, revoke_req).await;
    assert_eq!(revoke_resp.status(), 200, "Failed to revoke token");
    
    // Step 4: Validate again (should now be invalid)
    let validate_req2 = test::TestRequest::post()
        .uri("/api/tokens/validate")
        .set_json(&validate_req_body)
        .to_request();
    
    let validate_resp2 = test::call_service(&app, validate_req2).await;
    assert_eq!(validate_resp2.status(), 200);
    
    let response_body2: ValidateTokenResponse = test::read_body_json(validate_resp2).await;
    assert!(!response_body2.valid, "Token should be invalid after revocation");
}

#[actix_web::test]
async fn test_invalid_jwt_token() {
    let (db, app) = get_test_app().await;
    let invalid_jwt = "invalid.jwt.token";
    
    let validate_req_body = json!({
        "jwt_token": invalid_jwt
    });
    
    let validate_req = test::TestRequest::post()
        .uri("/api/tokens/validate")
        .set_json(&validate_req_body)
        .to_request();
    
    let validate_resp = test::call_service(&app, validate_req).await;
    
    assert_eq!(validate_resp.status(), 200, "Should return 200 even for invalid JWT");
    
    let response_body: ValidateTokenResponse = test::read_body_json(validate_resp).await;
    assert!(!response_body.valid, "Invalid JWT should not be valid");
}

#[actix_web::test]
async fn test_expired_jwt_token() {
    let (db, app) = get_test_app().await;
    let user = create_test_users(&db, 1).await.unwrap();
    let user_id = user[0];
    let expired_time = current_timestamp() - 3600; // 1 hour in the past
    let expired_jwt = create_test_jwt(&user_id, expired_time, None);
    
    let store_req_body = json!({
        "jwt_token": expired_jwt,
        "device_info": null
    });
    
    let store_req = test::TestRequest::post()
        .uri("/api/tokens/store")
        .set_json(&store_req_body)
        .to_request();
    
    let store_resp = test::call_service(&app, store_req).await;
    
    // Try to validate the expired token
    let validate_req_body = json!({
        "jwt_token": expired_jwt
    });
    
    let validate_req = test::TestRequest::post()
        .uri("/api/tokens/validate")
        .set_json(&validate_req_body)
        .to_request();
    
    let validate_resp = test::call_service(&app, validate_req).await;
    
    assert_eq!(validate_resp.status(), 200);
    
    let response_body: ValidateTokenResponse = test::read_body_json(validate_resp).await;
    assert!(!response_body.valid, "Expired JWT should not be valid");
}

#[actix_web::test]
async fn test_validation_request_with_invalid_input() {
    let (db, app) = get_test_app().await;
    
    // Try to validate with empty token
    let validate_req_body = json!({
        "jwt_token": ""
    });
    
    let validate_req = test::TestRequest::post()
        .uri("/api/tokens/validate")
        .set_json(&validate_req_body)
        .to_request();
    
    let validate_resp = test::call_service(&app, validate_req).await;
    
    assert_eq!(validate_resp.status(), 400, "Should return 400 for empty token");
}

#[actix_web::test]
async fn test_revocation_request_with_invalid_input() {
    let (db, app) = get_test_app().await;
    
    // Try to revoke with empty token
    let revoke_req_body = json!({
        "jwt_token": "",
        "reason": "Some reason"
    });
    
    let revoke_req = test::TestRequest::post()
        .uri("/api/tokens/revoke")
        .set_json(&revoke_req_body)
        .to_request();
    
    let revoke_resp = test::call_service(&app, revoke_req).await;
    
    assert_eq!(revoke_resp.status(), 400, "Should return 400 for empty token");
}

#[actix_web::test]
async fn test_revoke_non_existent_token() {
    let (db, app) = get_test_app().await;
    let user = create_test_users(&db, 1).await.unwrap();
    let user_id = user[0];
    let expiry = future_timestamp(3600);
    let jwt_token = create_test_jwt(&user_id, expiry, None);
    
    let revoke_req_body = json!({
        "jwt_token": jwt_token,
        "reason": "Revoking non-existent token"
    });
    
    let revoke_req = test::TestRequest::post()
        .uri("/api/tokens/revoke")
        .set_json(&revoke_req_body)
        .to_request();
    
    let revoke_resp = test::call_service(&app, revoke_req).await;
    
    assert!(
        revoke_resp.status().is_success(),
        "Unexpected status code: {}", revoke_resp.status()
    );
}

#[actix_web::test]
async fn test_multiple_tokens_for_same_user() {
    let (db, app) = get_test_app().await;
    let user = create_test_users(&db, 1).await.unwrap();
    let user_id = user[0];
    let expiry = future_timestamp(3600);
    
    let jwt_token1 = create_test_jwt(&user_id, expiry, Some("Device 1".to_string()));
    let jwt_token2 = create_test_jwt(&user_id, expiry, Some("Device 2".to_string()));
    
    let store_req_body1 = json!({
        "jwt_token": jwt_token1,
        "device_info": "Device 1"
    });
    
    let store_req1 = test::TestRequest::post()
        .uri("/api/tokens/store")
        .set_json(&store_req_body1)
        .to_request();
    
    let store_resp1 = test::call_service(&app, store_req1).await;
    assert_eq!(store_resp1.status(), 201, "Failed to store first token");
    
    let store_req_body2 = json!({
        "jwt_token": jwt_token2,
        "device_info": "Device 2"
    });
    
    let store_req2 = test::TestRequest::post()
        .uri("/api/tokens/store")
        .set_json(&store_req_body2)
        .to_request();
    
    let store_resp2 = test::call_service(&app, store_req2).await;
    assert_eq!(store_resp2.status(), 201, "Failed to store second token");
    
    // Revoke only the first token
    let revoke_req_body = json!({
        "jwt_token": jwt_token1,
        "reason": "First device logout"
    });
    
    let revoke_req = test::TestRequest::post()
        .uri("/api/tokens/revoke")
        .set_json(&revoke_req_body)
        .to_request();
    
    let revoke_resp = test::call_service(&app, revoke_req).await;
    assert_eq!(revoke_resp.status(), 200, "Failed to revoke first token");
    
    // First token should be invalid
    let validate_req_body1 = json!({
        "jwt_token": jwt_token1
    });
    
    let validate_req1 = test::TestRequest::post()
        .uri("/api/tokens/validate")
        .set_json(&validate_req_body1)
        .to_request();
    
    let validate_resp1 = test::call_service(&app, validate_req1).await;
    let response_body1: ValidateTokenResponse = test::read_body_json(validate_resp1).await;
    assert!(!response_body1.valid, "First token should be invalid after revocation");
    
    // Second token should still be valid
    let validate_req_body2 = json!({
        "jwt_token": jwt_token2
    });
    
    let validate_req2 = test::TestRequest::post()
        .uri("/api/tokens/validate")
        .set_json(&validate_req_body2)
        .to_request();
    
    let validate_resp2 = test::call_service(&app, validate_req2).await;
    let response_body2: ValidateTokenResponse = test::read_body_json(validate_resp2).await;
    assert!(response_body2.valid, "Second token should still be valid");
}
