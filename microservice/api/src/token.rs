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
use validator::{Validate, ValidationError};

#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct StoreTokenRequest {
    /// User ID for whom the token is being stored
    pub user_id: Uuid,
    
    /// Hashed token value
    #[validate(length(min = 1, max = 256, message = "Token hash must be between 1 and 256 characters"))]
    pub token_hash: String,
    
    /// Expiration timestamp (Unix timestamp)
    #[validate(range(min = 0, message = "Expiration timestamp must be a positive value"))]
    pub expires_at: i64,
    
    /// Optional device information associated with this token
    #[validate(length(max = 512, message = "Device info must not exceed 512 characters"))]
    pub device_info: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Validate)]
pub struct ValidateTokenRequest {
    /// User ID for token validation
    #[validate(custom = "validate_uuid")]
    pub user_id: Uuid,
    
    /// Hashed token to validate
    #[validate(length(min = 1, max = 256, message = "Token hash must be between 1 and 256 characters"))]
    pub token_hash: String,
}

#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct RevokeTokenRequest {
    /// Hashed token to revoke
    #[validate(length(min = 1, max = 256, message = "Token hash must be between 1 and 256 characters"))]
    pub token_hash: String,
    
    /// Optional reason for token revocation
    #[validate(length(max = 256, message = "Reason must not exceed 256 characters"))]
    pub reason: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ValidateTokenResponse {
    /// Indicates if the token is valid
    pub valid: bool,
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
}

impl<R: TokenRepository + 'static> TokenControllerImpl<R> {
    pub fn new(repository: R) -> Self {
        Self {
            service: TokenService::new(repository),
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
}

#[async_trait]
impl<R: TokenRepository> TokenController for TokenControllerImpl<R> {
    async fn store_token(&self, req: StoreTokenRequest) -> Result<(), AppError> {
        self.service
            .store_refresh_token(
                req.user_id,
                &req.token_hash,
                req.expires_at,
                req.device_info,
            )
            .await
    }

    async fn validate_token(&self, req: ValidateTokenRequest) -> Result<bool, AppError> {
        self.service
            .validate_refresh_token(req.user_id, &req.token_hash)
            .await
    }

    async fn revoke_token(&self, req: RevokeTokenRequest) -> Result<(), AppError> {
        self.service
            .revoke_refresh_token(&req.token_hash, req.reason)
            .await
    }
}

// Handler functions for actix-web

/// Validate the request before processing
fn validate_request<T: Validate>(req: &T) -> Result<(), AppError> {
    req.validate().map_err(|e| {
        AppError::ValidationError(format!("Invalid request: {}", e))
    })
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
async fn store_token_handler<R: TokenRepository>(
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
async fn validate_token_handler<R: TokenRepository>(
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
async fn revoke_token_handler<R: TokenRepository>(
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

    #[actix_web::test]
    async fn test_store_token_success() {
        // Setup
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_store_refresh_token()
            .with(
                always(),
                eq("test_hash"),
                eq(1234567890),
                eq(Some("test_device".to_string())),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::new_v4();
        let req = test::TestRequest::post()
            .uri("/api/tokens/store")
            .set_json(&StoreTokenRequest {
                user_id,
                token_hash: "test_hash".to_string(),
                expires_at: 1234567890,
                device_info: Some("test_device".to_string()),
            })
            .to_request();

        // Execute and verify
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }
    
    #[actix_web::test]
    async fn test_store_token_invalid_request() {
        // Setup
        let mock_repo = MockTokenRepository::new();
        
        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::new_v4();
        let req = test::TestRequest::post()
            .uri("/api/tokens/store")
            .set_json(&StoreTokenRequest {
                user_id,
                token_hash: "".to_string(), // Invalid empty token
                expires_at: -100,  // Invalid negative timestamp
                device_info: Some("test_device".to_string()),
            })
            .to_request();

        // Execute and verify
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_validate_token_success() {
        // Setup
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_validate_refresh_token()
            .with(always(), eq("test_hash"))
            .times(1)
            .returning(|_, _| Ok(true));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::new_v4();
        let req = test::TestRequest::post()
            .uri("/api/tokens/validate")
            .set_json(&ValidateTokenRequest {
                user_id,
                token_hash: "test_hash".to_string(),
            })
            .to_request();

        // Execute and verify
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let validate_resp: ValidateTokenResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(validate_resp.valid, true);
    }
    
    #[actix_web::test]
    async fn test_validate_token_invalid() {
        // Setup
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_validate_refresh_token()
            .with(always(), eq("test_hash"))
            .times(1)
            .returning(|_, _| Ok(false));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::new_v4();
        let req = test::TestRequest::post()
            .uri("/api/tokens/validate")
            .set_json(&ValidateTokenRequest {
                user_id,
                token_hash: "test_hash".to_string(),
            })
            .to_request();

        // Execute and verify
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let validate_resp: ValidateTokenResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(validate_resp.valid, false);
    }
    
    #[actix_web::test]
    async fn test_validate_token_invalid_request() {
        // Setup
        let mock_repo = MockTokenRepository::new();
        
        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::new_v4();
        let req = test::TestRequest::post()
            .uri("/api/tokens/validate")
            .set_json(&ValidateTokenRequest {
                user_id,
                token_hash: "".to_string(), // Invalid empty token
            })
            .to_request();

        // Execute and verify
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_revoke_token_success() {
        // Setup
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_revoke_refresh_token()
            .with(eq("test_hash"), eq(Some("expired".to_string())))
            .times(1)
            .returning(|_, _| Ok(()));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/tokens/revoke")
            .set_json(&RevokeTokenRequest {
                token_hash: "test_hash".to_string(),
                reason: Some("expired".to_string()),
            })
            .to_request();

        // Execute and verify
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_revoke_token_no_reason() {
        // Setup
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_revoke_refresh_token()
            .with(eq("test_hash"), eq(None))
            .times(1)
            .returning(|_, _| Ok(()));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/tokens/revoke")
            .set_json(&RevokeTokenRequest {
                token_hash: "test_hash".to_string(),
                reason: None,
            })
            .to_request();

        // Execute and verify
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_revoke_token_invalid_request() {
        // Setup
        let mock_repo = MockTokenRepository::new();
        
        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/tokens/revoke")
            .set_json(&RevokeTokenRequest {
                token_hash: "".to_string(), // Invalid empty token
                reason: Some("expired".to_string()),
            })
            .to_request();

        // Execute and verify
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
    
    #[actix_web::test]
    async fn test_repository_error_handling() {
        // Setup
        let mut mock_repo = MockTokenRepository::new();
        mock_repo
            .expect_validate_refresh_token()
            .with(always(), eq("test_hash"))
            .times(1)
            .returning(|_, _| Err(AppError::DatabaseError("DB connection failed".to_string())));

        let app = test::init_service(
            App::new().configure(TokenControllerImpl::configure(mock_repo)),
        ).await;

        let user_id = Uuid::new_v4();
        let req = test::TestRequest::post()
            .uri("/api/tokens/validate")
            .set_json(&ValidateTokenRequest {
                user_id,
                token_hash: "test_hash".to_string(),
            })
            .to_request();

        // Execute and verify
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
