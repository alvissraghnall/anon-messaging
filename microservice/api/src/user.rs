use db::{
    faker_rand::en_us::names::FullName,
    models::User,
    uuid::{self, Uuid},
};
use mockall::automock;
use service::user::{UserRepository, UserService};
use shared::{
    errors::AppError,
    models::{RegisterRequest, RegisterResponse, UpdateUserRequest},
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use actix_web::{
    delete, get, patch, post,
    web::{self, Data, Json, Path, Query},
    HttpResponse, Responder,
};
use utoipa::ToSchema;
use validator::Validate;
use std::collections::HashMap;


pub type RegisterUserResponse = Result<HttpResponse, AppError>;
pub type GetUserResponse = Result<HttpResponse, AppError>;
pub type GetUsersResponse = Result<HttpResponse, AppError>;
pub type UpdateUserResponse = Result<HttpResponse, AppError>;
pub type DeleteUserResponse = Result<HttpResponse, AppError>;


/// A single field validation error
#[derive(Serialize, ToSchema)]
pub struct FieldValidationErrorDoc {
    /// The validation code (e.g., "length", "email")
    pub code: String,
    /// Optional human-readable message
    pub message: Option<String>,
}

/// Error response returned when validation fails
#[derive(Serialize, ToSchema)]
pub struct ValidationErrorResponseDoc {
    /// A map of field names to their validation errors
    pub errors: HashMap<String, Vec<FieldValidationErrorDoc>>,
}

#[derive(Deserialize)]
struct GetUsersQuery {
    pub limit: Option<i64>,
}

#[automock]
#[async_trait::async_trait]
pub trait UserController: Send + Sync {
    async fn register_user(&self, request: Json<RegisterRequest>) -> RegisterUserResponse;

    async fn get_user(self: &Self, user_id: Path<Uuid>) -> GetUserResponse;

    async fn get_users(self: &Self, limit: Query<Option<i64>>) -> GetUsersResponse;

    async fn update_user(
        self: &Self,
        user_id: Path<Uuid>,
        request: Json<UpdateUserRequest>,
    ) -> UpdateUserResponse;

    /*
    async fn delete_user(self: &Self, user_id: Path<Uuid>) -> DeleteUserResponse;
    */
}

pub struct UserControllerImpl<R: UserRepository> {
    service: Data<UserService<R>>,
}

impl<R: UserRepository> UserControllerImpl<R> {
    pub fn new(service: Data<UserService<R>>) -> Self {
        Self { service }
    }
}

#[async_trait::async_trait]
impl<R: UserRepository + 'static> UserController for UserControllerImpl<R> {
    async fn register_user(self: &Self, request: Json<RegisterRequest>) -> RegisterUserResponse {
        let response = self.service.register_user(request.into_inner()).await?;
        Ok(HttpResponse::Created().json(response))
    }

    async fn get_user(self: &Self, user_id: Path<Uuid>) -> GetUserResponse {
        let user = self.service.get_user_by_id(*user_id).await?;
        Ok(HttpResponse::Ok().json(user))
    }

    async fn get_users(self: &Self, limit: Query<Option<i64>>) -> GetUsersResponse {
        let users = self.service.get_users(limit.into_inner()).await?;
        Ok(HttpResponse::Ok().json(users))
    }

    async fn update_user(
        self: &Self,
        user_id: Path<Uuid>,
        request: Json<UpdateUserRequest>,
    ) -> UpdateUserResponse {
        self.service
            .update_user(*user_id, request.into_inner())
            .await?;
        Ok(HttpResponse::Ok().finish())
    }

    /*
    #[utoipa::path(
        delete,
        path = "/{user_id}",
        params(
            ("user_id" = Uuid, Path, description = "User ID")
        ),
        responses(
            (status = 204, description = "User deleted successfully"),
            (status = 404, description = "User not found"),
            (status = 500, description = "Internal server error"),
        )
    )]
    async fn delete_user(&self, user_id: Path<Uuid>) -> DeleteUserResponse {
        self.service.delete_user(*user_id).await?;
        Ok(HttpResponse::NoContent().finish())
    } */
}

// Actix-web route handlers
#[utoipa::path(
    post,
    path = "",
    request_body(content = RegisterRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Validation error", body = ValidationErrorResponseDoc),
        (status = 409, description = "Username or public key already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
#[post("")]
pub async fn register_user_handler(
    controller: Data<Arc<dyn UserController>>,
    request: Json<RegisterRequest>,
) -> impl Responder {
    println!("{}", "god. \n");
    controller.register_user(request).await
}

#[utoipa::path(
    get,
    path = "/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[get("/{user_id}")]
pub async fn get_user_handler(
    controller: Data<Arc<dyn UserController>>,
    user_id: Path<Uuid>,
) -> impl Responder {
    controller.get_user(user_id).await
}

#[utoipa::path(
    get,
    path = "",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of users to return")
    ),
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 500, description = "Internal server error"),
    )
)]
#[get("")]
pub async fn get_users_handler(
    controller: Data<Arc<dyn UserController>>,
    limit: Query<GetUsersQuery>,
) -> impl Responder {
    controller.get_users(Query(limit.limit)).await
}

#[utoipa::path(
    patch,
    path = "/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully"),
        (status = 400, description = "Validation error"),
        (status = 404, description = "User not found"),
        (status = 409, description = "New username already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
#[patch("/{user_id}")]
pub async fn update_user_handler(
    controller: Data<Arc<dyn UserController>>,
    user_id: Path<Uuid>,
    request: Json<UpdateUserRequest>,
) -> impl Responder {
    controller.update_user(user_id, request).await
}

/*
#[delete("/{user_id}")]
pub async fn delete_user_handler(
    controller: Data<Arc<dyn UserController>>,
    user_id: Path<Uuid>,
) -> impl Responder {
    controller.delete_user(user_id).await
}
*/

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/users")
            .service(register_user_handler)
            .service(get_user_handler)
            .service(get_users_handler)
            .service(update_user_handler),
        // .service(delete_user_handler),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{body::to_bytes, test, ResponseError};
    use base64::{
        alphabet,
        engine::{self, general_purpose},
        Engine as _,
    };
    use db::public_key::PublicKey;
    use db::public_key_hash::PublicKeyHash;
    use mockall::predicate::*;
    use mockall::{mock, predicate::*};
    use service::p256::{
        ecdsa::{SigningKey, VerifyingKey},
        elliptic_curve::rand_core::OsRng,
    };
    use service::rand::Rng;
    use shared::crypto::utils::sha256_hash;
    use sqlx::types::chrono::Utc;
    use service::user::MockUserRepository;

    const CUSTOM_ENGINE: engine::GeneralPurpose =
        engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);


    impl Clone for MockUserController {
        fn clone(&self) -> Self {
            Self::default()
        }
    }

    fn setup_mock_controller() -> (Data<Arc<MockUserController>>, MockUserController) {
        let mock_controller = MockUserController::new();
        let controller_arc = Arc::new(mock_controller.clone());
        (Data::new(controller_arc), mock_controller)
    }

    async fn setup_controller() -> (
        Data<UserControllerImpl<MockUserRepository>>,
        MockUserRepository,
    ) {
        let mock_repo = MockUserRepository::new();
        let service = Data::new(UserService::new(mock_repo.clone()));
        let controller = Data::new(UserControllerImpl::new(service));
        (controller, mock_repo)
    }
    
    /*
        fn setup_mock_controller() -> (Data<Arc<MockUserController>>, MockUserServiceTrait) {
            let mock_service = MockUserServiceTrait::new();
            let controller = Data::new(Arc::new(MockUserController::new()));
            (controller, mock_service)
        }
    */
    async fn parse_response_body<T: serde::de::DeserializeOwned>(response: HttpResponse) -> T {
        let body = to_bytes(response.into_body()).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    pub async fn generate_key() -> (PublicKey, PublicKeyHash) {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);

        let b64_key = CUSTOM_ENGINE.encode(verifying_key.to_encoded_point(true).as_bytes());
        let public_key = PublicKey::new(b64_key).unwrap();

        let public_key_hash = public_key.to_hash().unwrap();

        (public_key, public_key_hash)
    }

    #[actix_web::test]
    async fn test_register_user_success() {
        let mut mock_repo = MockUserRepository::new();

        let test_uuid = Uuid::now_v7();
        let (public_key, public_key_hash) = generate_key().await;
        
        let request = RegisterRequest {
            username: Some("testuser".to_string()),
            public_key: public_key.to_string(),
        };

        mock_repo
            .expect_insert_user()
            .withf(move |pk, username| {
                username == "testuser".to_string() && 
                pk.to_string() == public_key.to_string()
            })
            .times(1)
            .returning(move |username, _| {
                Ok(
                    test_uuid
                )
            });

        let service = Data::new(UserService::new(mock_repo));
        let controller = Data::new(UserControllerImpl::new(service));
       
        let response = controller.register_user(Json(request)).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        
        let body: RegisterResponse = parse_response_body(response).await;
        assert_eq!(body.user_id, test_uuid);
        assert_eq!(body.username, "testuser");
    }

    #[actix_web::test]
    async fn test_register_user_without_username() {
        let mut mock_repo = MockUserRepository::new();

        let test_uuid = Uuid::now_v7();
        let request = RegisterRequest {
            username: None,
            public_key: "test_public_key".to_string(),
        };
        let req_json = Json(request.clone());

        let expected_response = RegisterResponse {
            user_id: test_uuid,
            username: "testuser".to_string(),
        };

        mock_repo
            .expect_insert_user()
            .withf(move |pk, username| {
                pk == "test_public_key"
            })
            .times(1)
            .returning(move |username, _| {
                Ok(
                    test_uuid
                )
            });


        let service = Data::new(UserService::new(mock_repo));
        let controller = Data::new(UserControllerImpl::new(service));

        let response = controller.register_user(Json(request)).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn test_get_user_success() {
        let mut mock_repo = MockUserRepository::new();
        let (public_key, public_key_hash) = generate_key().await;

        let test_uuid = Uuid::now_v7();
        let test_user = User {
            id: test_uuid,
            username: "testuser".to_string(),
            public_key,
            public_key_hash,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            last_login: Some(Utc::now().naive_utc()),
        };

        mock_repo.expect_get_user_by_id()
            .with(eq(test_uuid))
            .times(1)
            .returning(move |_| Ok(test_user.clone()));

        let service = Data::new(UserService::new(mock_repo));
        let controller = Data::new(UserControllerImpl::new(service));

        let response = controller.get_user(Path::from(test_uuid)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body: User = parse_response_body(response).await;
        assert_eq!(body.id, test_uuid);
        assert_eq!(body.username, "testuser");
    }

    #[actix_web::test]
    async fn test_get_users_success() {
        let mut mock_1 = MockUserRepository::new();
        let mut mock_2 = MockUserRepository::new();
        let (public_key, public_key_hash) = generate_key().await;
        let (public_key2, public_key_hash2) = generate_key().await;

        let test_users = vec![
            User {
                id: Uuid::now_v7(),
                username: "user1".to_string(),
                public_key: public_key,
                public_key_hash: public_key_hash,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
                last_login: Some(Utc::now().naive_utc()),
            },
            User {
                id: Uuid::now_v7(),
                username: "user2".to_string(),
                public_key: public_key2,
                public_key_hash: public_key_hash2,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
                last_login: Some(Utc::now().naive_utc()),
            },
        ];

        let test_users_1 = test_users.clone();
        let test_users_2 = test_users.clone();

        // Test with limit
        mock_1
            .expect_get_users()
            .with(eq(Some(10)))
            .times(1)
            .returning(move |_| Ok(test_users_1.clone()));

        let service_1 = Data::new(UserService::new(mock_1));
        let controller_1 = Data::new(UserControllerImpl::new(service_1));

        let response = controller_1
            .get_users(Query::from(actix_web::web::Query(Some(10))))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body: Vec<User> = parse_response_body(response).await;
        assert_eq!(body.len(), 2);

        // Test without limit
        mock_2
            .expect_get_users()
            .with(eq(None))
            .times(1)
            .returning(move |_| Ok(test_users_2.clone()));

        let service_2 = Data::new(UserService::new(mock_2));
        let controller_2 = Data::new(UserControllerImpl::new(service_2));

        let response = controller_2
            .get_users(Query::from(actix_web::web::Query(None)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_update_user_success() {
        let mut mock = MockUserRepository::new();
        let test_uuid = Uuid::now_v7();
        let request = UpdateUserRequest {
            new_username: Some("newusername".to_string()),
            new_public_key: None,
        };
        let req_json = Json(request.clone());

        mock.expect_update_user()
            .with(
                eq(test_uuid),
                eq(Some("newusername".to_string())),
                eq(None),
                eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let service = Data::new(UserService::new(mock));
        let controller = Data::new(UserControllerImpl::new(service));

        let response = controller
            .update_user(Path::from(test_uuid), Json(request))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_update_user_with_public_key() {
        let mut mock = MockUserRepository::new();
        let (public_key, public_key_hash) = generate_key().await;
        let encoded_pkhash = CUSTOM_ENGINE.encode(public_key_hash.to_string());
        println!("{} {} {} {}", public_key.to_string(), public_key_hash.to_string(), encoded_pkhash, CUSTOM_ENGINE.encode(public_key.to_string()));

        let test_uuid = Uuid::now_v7();
        let new_key = "new_public_key".to_string();
        let request = UpdateUserRequest {
            new_username: None,
            new_public_key: Some(public_key.to_string()),
        };

        let req_json = Json(request.clone());

        mock.expect_update_user()
            .with(
                eq(test_uuid),
                eq(None),
                eq(Some(public_key.to_string())),
                always(),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));


        let service = Data::new(UserService::new(mock));
        let controller = Data::new(UserControllerImpl::new(service));

        let response = controller
            .update_user(Path::from(test_uuid), Json(request))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_register_user_validation_errors() {
        let request = RegisterRequest {
            username: Some("ab".to_string()),
            public_key: "test_public_key".to_string(),
        };
        let validation = request.validate();
        assert!(validation.is_err());
        let errors = validation.unwrap_err();
        assert!(errors.field_errors().contains_key("username"));

        let request = RegisterRequest {
            username: Some("valid".to_string()),
            public_key: "short".to_string(),
        };
        let validation = request.validate();
        assert!(validation.is_err());
        let errors = validation.unwrap_err();
        assert!(errors.field_errors().contains_key("public_key"));
    }

    #[actix_web::test]
    async fn test_get_user_not_found() {
        let mut mock = MockUserRepository::new();
        let test_uuid = Uuid::now_v7();

        mock.expect_get_user_by_id()
            .with(eq(test_uuid))
            .times(1)
            .returning(|_| Err(AppError::NotFound("User not found".to_string())));

        let service = Data::new(UserService::new(mock));
        let controller = Data::new(UserControllerImpl::new(service));
        
        let response = controller.get_user(Path::from(test_uuid)).await;
        assert!(response.is_err());

        if let Err(err) = response {
            assert_eq!(err.error_response().status(), StatusCode::NOT_FOUND);
            assert_eq!(err.to_string(), "Not found: User not found");
        }
    }

    #[actix_web::test]
    async fn test_register_user_conflict() {
        let mut mock = MockUserRepository::new();

        let request = RegisterRequest {
            username: Some("existing_user".to_string()),
            public_key: "existing_key".to_string(),
        };

        mock.expect_insert_user()
            .with(
                eq("existing_key".to_string()),
                eq("existing_user".to_string())
            )
            .times(1)
            .returning(|_, _| {
                Err(AppError::UniqueViolation(
                    "Username already exists".to_string(),
                ))
            });

        let service = Data::new(UserService::new(mock));
        let controller = Data::new(UserControllerImpl::new(service));

        let response = controller.register_user(Json(request)).await;
        assert!(response.is_err());

        if let Err(err) = response {
            assert_eq!(err.error_response().status(), StatusCode::CONFLICT);
            assert_eq!(
                err.to_string(),
                "Unique constraint violation: Username already exists"
            );
        }
    }

    #[actix_web::test]
    async fn test_update_user_validation() {
        let mut mock = MockUserRepository::new();
        let test_uuid = Uuid::now_v7();

        let invalid_request = UpdateUserRequest {
            new_username: Some("ab".to_string()),
            new_public_key: None,
        };

        let validation = invalid_request.validate();
        assert!(validation.is_err());

        let invalid_request = UpdateUserRequest {
            new_username: None,
            new_public_key: Some("short".to_string()),
        };

        let validation = invalid_request.validate();
        assert!(validation.is_err());
    }

    #[actix_web::test]
    async fn test_update_user_not_found() {
        let mut mock = MockUserRepository::new();
        let test_uuid = Uuid::now_v7();
        let request = UpdateUserRequest {
            new_username: Some("newusername".to_string()),
            new_public_key: None,
        };

        mock.expect_update_user()
            .with(always(), always(), always(), always())
            .times(1)
            .returning(|_, _, _, _| Err(AppError::NotFound("User not found".to_string())));

        let service = Data::new(UserService::new(mock));
        let controller = Data::new(UserControllerImpl::new(service));
        
        let response = controller
            .update_user(Path::from(test_uuid), Json(request))
            .await;

        assert!(response.is_err());
        if let Err(err) = response {
            assert_eq!(err.error_response().status(), StatusCode::NOT_FOUND);
        }
    }
}
