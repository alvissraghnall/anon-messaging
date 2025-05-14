use actix_web::{web, HttpResponse, Responder};
use db::uuid::Uuid;
use serde::{Deserialize, Serialize};
use shared::errors::AppError;
use utoipa::{IntoParams, ToSchema};
use utoipa::OpenApi;
use service::token::repository::TokenRepository;
use service::token::service::TokenService;
use async_trait::async_trait;
use mockall::automock;
use validator::{Validate, ValidationError, ValidationErrors};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, TokenData};
use std::time::{SystemTime, UNIX_EPOCH};
use service::sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    /// Subject (user ID)
    sub: String,
    /// Expiration time (Unix timestamp)
    exp: i64,
    /// Issued at time (Unix timestamp)
    iat: i64,
    /// Optional device information
    #[serde(skip_serializing_if = "Option::is_none")]
    device: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct StoreTokenRequest {
    /// JWT token to store
    #[validate(length(min = 1, message = "JWT token must not be empty"))]
    pub jwt_token: String,
    
    /// Optional device information associated with this token
    #[validate(length(max = 512, message = "Device info must not exceed 512 characters"))]
    pub device_info: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Validate)]
pub struct ValidateTokenRequest {
    /// JWT token to validate
    #[validate(length(min = 1, message = "JWT token must not be empty"))]
    pub jwt_token: String,
}

#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct RevokeTokenRequest {
    /// JWT token to revoke
    #[validate(length(min = 1, message = "JWT token must not be empty"))]
    pub jwt_token: String,
    
    /// Optional reason for token revocation
    #[validate(length(max = 256, message = "Reason must not exceed 256 characters"))]
    pub reason: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ValidateTokenResponse {
    /// Indicates if the token is valid
    pub valid: bool,
}

/// Configuration for JWT verification
pub struct JwtConfig {
    /// Decoding key for JWT verification
    pub decoding_key: DecodingKey,
    /// JWT validation settings
    pub validation: Validation,
}

impl Default for JwtConfig {
    fn default() -> Self {
        // In a real application, this would be loaded from environment or configuration
        let secret = "your_jwt_secret_key";
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        
        Self {
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            validation,
        }
    }
}

/// Verifies JWT token and extracts claims
fn verify_jwt(token: &str, config: &JwtConfig) -> Result<TokenData<JwtClaims>, AppError> {
    Ok(decode::<JwtClaims>(token, &config.decoding_key, &config.validation)?)
}

#[automock]
#[async_trait]
pub trait TokenController: Send + Sync {
    async fn store_token(&self, req: StoreTokenRequest) -> Result<(), AppError>;
    async fn validate_token(&self, req: ValidateTokenRequest) -> Result<bool, AppError>;
    async fn revoke_token(&self, req: RevokeTokenRequest) -> Result<(), AppError>;
}

pub struct TokenControllerImpl<R: TokenRepository> {
    service: TokenService<R>,
    jwt_config: JwtConfig,
}

impl<R: TokenRepository + 'static> TokenControllerImpl<R> {
    pub fn new(repository: R) -> Self {
        Self {
            service: TokenService::new(repository),
            jwt_config: JwtConfig::default(),
        }
    }
    
    pub fn new_with_config(repository: R, jwt_config: JwtConfig) -> Self {
        Self {
            service: TokenService::new(repository),
            jwt_config,
        }
    }

    /// Configure routes for the token controller
    pub fn configure(repository: R) -> impl FnOnce(&mut web::ServiceConfig) {
        let controller = Self::new(repository);

        |cfg: &mut web::ServiceConfig| {
            cfg.app_data(web::JsonConfig::default().limit(4096))
                .service(
                    web::scope("/api/tokens")
                        .route("/store", web::post().to(store_token_handler::<R>))
                        .route("/validate", web::post().to(validate_token_handler::<R>))
                        .route("/revoke", web::post().to(revoke_token_handler::<R>))
                        .app_data(web::Data::new(controller))
                );
        }
    }
    
    /// Generate a hash from JWT token for storage
    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[async_trait]
impl<R: TokenRepository + 'static> TokenController for TokenControllerImpl<R> {
    async fn store_token(&self, req: StoreTokenRequest) -> Result<(), AppError> {
        // Verify JWT and extract claims
        let token_data = verify_jwt(&req.jwt_token, &self.jwt_config)?;
        let claims = token_data.claims;
        
        // Parse user ID from subject claim
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| {
                let mut errors = ValidationErrors::new();

                let mut error = ValidationError::new("invalid_user_id");
                error.message = Some("Invalid user ID in JWT".into());

                errors.add("user_id", error);

                AppError::ValidationError(errors)
            })?;
            
        // Hash the token for storage
        let token_hash = self.hash_token(&req.jwt_token);
        
        // Use device info from request or JWT claims
        let device_info = req.device_info.or(claims.device);
        
        self.service
            .store_refresh_token(
                user_id,
                &token_hash,
                claims.exp,
                device_info,
            )
            .await
    }

    async fn validate_token(&self, req: ValidateTokenRequest) -> Result<bool, AppError> {
        // First verify JWT structure and signature
        let jwt_validation = verify_jwt(&req.jwt_token, &self.jwt_config);
        
        match jwt_validation {
            // If JWT is valid, check if it's in the database and not revoked
            Ok(token_data) => {
                let user_id = Uuid::parse_str(&token_data.claims.sub)
                    .map_err(|_| {
                        let mut errors = ValidationErrors::new();

                        let mut error = ValidationError::new("invalid_user_id");
                        error.message = Some("Invalid user ID in JWT".into());

                        errors.add("user_id", error);

                        AppError::ValidationError(errors)
                    })?;
                    
                let token_hash = self.hash_token(&req.jwt_token);
                
                // Check if token is in repository and not revoked
                self.service
                    .validate_refresh_token(user_id, &token_hash)
                    .await
            },
            // If JWT is invalid, return false without checking database
            Err(_) => Ok(false),
        }
    }

    async fn revoke_token(&self, req: RevokeTokenRequest) -> Result<(), AppError> {
        let token_hash = self.hash_token(&req.jwt_token);
        
        self.service
            .revoke_refresh_token(&token_hash, req.reason)
            .await
    }
}

// Handler functions for actix-web

/// Validate the request before processing
fn validate_request<T: Validate>(req: &T) -> Result<(), AppError> {
    Ok(req.validate()?)
}

#[utoipa::path(
    post,
    path = "/api/tokens/store",
    request_body = StoreTokenRequest,
    responses(
        (status = 201, description = "Token stored successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "tokens"
)]
async fn store_token_handler<R: TokenRepository + 'static>(
    controller: web::Data<TokenControllerImpl<R>>,
    req: web::Json<StoreTokenRequest>,
) -> Result<impl Responder, AppError> {
    let req = req.into_inner();
    
    // Validate request
    validate_request(&req)?;
    
    controller.store_token(req).await?;
    Ok(HttpResponse::Created().finish())
}

#[utoipa::path(
    post,
    path = "/api/tokens/validate",
    request_body = ValidateTokenRequest,
    responses(
        (status = 200, description = "Token validated", body = ValidateTokenResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "tokens"
)]
async fn validate_token_handler<R: TokenRepository + 'static>(
    controller: web::Data<TokenControllerImpl<R>>,
    req: web::Json<ValidateTokenRequest>,
) -> Result<impl Responder, AppError> {
    let req = req.into_inner();
    
    // Validate request
    validate_request(&req)?;
    
    let is_valid = controller.validate_token(req).await?;
    Ok(HttpResponse::Ok().json(ValidateTokenResponse { valid: is_valid }))
}

#[utoipa::path(
    post,
    path = "/api/tokens/revoke",
    request_body = RevokeTokenRequest,
    responses(
        (status = 200, description = "Token revoked successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "tokens"
)]
async fn revoke_token_handler<R: TokenRepository + 'static>(
    controller: web::Data<TokenControllerImpl<R>>,
    req: web::Json<RevokeTokenRequest>,
) -> Result<impl Responder, AppError> {
    let req = req.into_inner();
    
    // Validate request
    validate_request(&req)?;
    
    controller.revoke_token(req).await?;
    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App, http::StatusCode};
    use mockall::predicate::*;
    use service::token::repository::MockTokenRepository;
    use jsonwebtoken::{encode, EncodingKey, Header};

    fn create_test_jwt(user_id: &Uuid, expires_in_secs: i64, device_info: Option<String>) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
            
        let claims = JwtClaims {
            sub: user_id.to_string(),
            exp: now + expires_in_secs,
            iat: now,
            device: device_info,
        };
        
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("your_jwt_secret_key".as_bytes())
        ).unwrap()
    }

    #[actix_web::test]
    async fn test_store_token_success() {
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_store_refresh_token()
            .with(
                always(),
                always(),
                gt(0),
                eq(Some("test_device".to_string())),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::now_v7();
        let jwt_token = create_test_jwt(&user_id, 3600, Some("test_device".to_string()));
        
        let req = test::TestRequest::post()
            .uri("/api/tokens/store")
            .set_json(&StoreTokenRequest {
                jwt_token,
                device_info: Some("test_device".to_string()),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }
    
    #[actix_web::test]
    async fn test_store_token_invalid_jwt() {
        let mock_repo = MockTokenRepository::new();
        
        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/tokens/store")
            .set_json(&StoreTokenRequest {
                jwt_token: "invalid.jwt.token".to_string(),
                device_info: Some("test_device".to_string()),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
    
    #[actix_web::test]
    async fn test_store_token_empty_jwt() {
        let mock_repo = MockTokenRepository::new();
        
        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/tokens/store")
            .set_json(&StoreTokenRequest {
                jwt_token: "".to_string(),
                device_info: Some("test_device".to_string()),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_validate_token_success() {
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_validate_refresh_token()
            .with(always(), always())
            .times(1)
            .returning(|_, _| Ok(true));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::now_v7();
        let jwt_token = create_test_jwt(&user_id, 3600, None);
        
        let req = test::TestRequest::post()
            .uri("/api/tokens/validate")
            .set_json(&ValidateTokenRequest {
                jwt_token,
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let validate_resp: ValidateTokenResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(validate_resp.valid, true);
    }
    
    #[actix_web::test]
    async fn test_validate_token_expired_jwt() {
        let mock_repo = MockTokenRepository::new();
        
        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::now_v7();
        let jwt_token = create_test_jwt(&user_id, -3600, None); // expired 1 hour ago
        
        let req = test::TestRequest::post()
            .uri("/api/tokens/validate")
            .set_json(&ValidateTokenRequest {
                jwt_token,
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let validate_resp: ValidateTokenResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(validate_resp.valid, false);
    }
    
    #[actix_web::test]
    async fn test_validate_token_valid_jwt_but_revoked() {
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_validate_refresh_token()
            .with(always(), always())
            .times(1)
            .returning(|_, _| Ok(false));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::now_v7();
        let jwt_token = create_test_jwt(&user_id, 3600, None);
        
        let req = test::TestRequest::post()
            .uri("/api/tokens/validate")
            .set_json(&ValidateTokenRequest {
                jwt_token,
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let validate_resp: ValidateTokenResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(validate_resp.valid, false);
    }

    #[actix_web::test]
    async fn test_revoke_token_success() {
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_revoke_refresh_token()
            .with(always(), eq(Some("expired".to_string())))
            .times(1)
            .returning(|_, _| Ok(()));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::now_v7();
        let jwt_token = create_test_jwt(&user_id, 3600, None);
        
        let req = test::TestRequest::post()
            .uri("/api/tokens/revoke")
            .set_json(&RevokeTokenRequest {
                jwt_token,
                reason: Some("expired".to_string()),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_revoke_invalid_token() {
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_revoke_refresh_token()
            .with(always(), eq(None))
            .times(1)
            .returning(|_, _| Ok(()));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/tokens/revoke")
            .set_json(&RevokeTokenRequest {
                jwt_token: "invalid.but.still.hashable".to_string(),
                reason: None,
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_repository_error_handling() {
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_validate_refresh_token()
            .with(always(), always())
            .times(1)
            .returning(|_, _| Err(AppError::DatabaseError(db::Error::InvalidArgument("DB connection failed".to_string()))));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::now_v7();
        let jwt_token = create_test_jwt(&user_id, 3600, None);
        
        let req = test::TestRequest::post()
            .uri("/api/tokens/validate")
            .set_json(&ValidateTokenRequest {
                jwt_token,
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    #[actix_web::test]
    async fn test_revoke_token_no_reason() {
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_revoke_refresh_token()
            .with(always(), eq(None))
            .times(1)
            .returning(|_, _| Ok(()));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::now_v7();
        let jwt_token = create_test_jwt(&user_id, 3600, None);

        let req = test::TestRequest::post()
            .uri("/api/tokens/revoke")
            .set_json(&RevokeTokenRequest {
                jwt_token,
                reason: None,
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_revoke_token_invalid_request() {
        let mut mock_repo = MockTokenRepository::new();

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::now_v7();
        let jwt_token = create_test_jwt(&user_id, 3600, None);
        
        let req = test::TestRequest::post()
            .uri("/api/tokens/revoke")
            .set_json(&RevokeTokenRequest {
                jwt_token: String::from(""),
                reason: Some("expired".to_string()),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
    
}
