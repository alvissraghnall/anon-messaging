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

use actix_web::{
    delete, get, patch, post,
    web::{self, Data, Json, Path, Query},
    HttpResponse, Responder,
};
use utoipa::ToSchema;

pub type RegisterUserResponse = Result<HttpResponse, AppError>;
pub type GetUserResponse = Result<HttpResponse, AppError>;
pub type GetUsersResponse = Result<HttpResponse, AppError>;
pub type UpdateUserResponse = Result<HttpResponse, AppError>;
pub type DeleteUserResponse = Result<HttpResponse, AppError>;

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
        path = "/api/users/{user_id}",
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
    path = "/api/users",
    request_body(content = RegisterRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Username or public key already exists"),
        (status = 500, description = "Internal server error"),
    )
)]
#[post("/api/users")]
pub async fn register_user_handler(
    controller: Data<Arc<dyn UserController>>,
    request: Json<RegisterRequest>,
) -> impl Responder {
    controller.register_user(request).await
}

#[utoipa::path(
    get,
    path = "/api/users/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[get("/api/users/{user_id}")]
pub async fn get_user_handler(
    controller: Data<Arc<dyn UserController>>,
    user_id: Path<Uuid>,
) -> impl Responder {
    controller.get_user(user_id).await
}

#[utoipa::path(
    get,
    path = "/api/users",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of users to return")
    ),
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 500, description = "Internal server error"),
    )
)]
#[get("/api/users")]
pub async fn get_users_handler(
    controller: Data<Arc<dyn UserController>>,
    limit: Query<Option<i64>>,
) -> impl Responder {
    controller.get_users(limit).await
}

#[utoipa::path(
    patch,
    path = "/api/users/{user_id}",
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
#[patch("/api/users/{user_id}")]
pub async fn update_user_handler(
    controller: Data<Arc<dyn UserController>>,
    user_id: Path<Uuid>,
    request: Json<UpdateUserRequest>,
) -> impl Responder {
    controller.update_user(user_id, request).await
}

/*
#[delete("/api/users/{user_id}")]
pub async fn delete_user_handler(
    controller: Data<Arc<dyn UserController>>,
    user_id: Path<Uuid>,
) -> impl Responder {
    controller.delete_user(user_id).await
}
*/

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(register_user_handler)
            .service(get_user_handler)
            .service(get_users_handler)
            .service(update_user_handler), // .service(delete_user_handler),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use actix_web::http::StatusCode;
    use sqlx::types::chrono::Utc;
    use actix_web::{test, body::to_bytes, ResponseError};
    use shared::crypto::utils::sha256_hash;
    use mockall::mock;

    mock! {
        pub UserRepoImpl {}
        #[async_trait::async_trait]
        impl UserRepository for UserRepoImpl {
            async fn insert_user(
                &self,
                public_key_hash: &str,
                public_key: &str,
                username: &str,
            ) -> Result<Uuid, AppError>;
            
            async fn get_user_by_pubkey(&self, public_key_hash: &str) -> Result<User, AppError>;
            async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, AppError>;
            async fn get_users(&self, limit: Option<i64>) -> Result<Vec<User>, AppError>;
            async fn update_user(
                &self,
                user_id: Uuid,
                new_username: Option<String>,
                new_public_key: Option<String>,
                new_public_key_hash: Option<String>,
            ) -> Result<(), AppError>;
            async fn fetch_public_key_hash(&self, user_id: Uuid) -> Result<String, AppError>;
        }
    }

    
    impl Clone for MockUserRepoImpl {
        fn clone(&self) -> Self {
            Self::default()
        }
    }

    fn setup_mock_service() -> (Data<UserService<MockUserRepoImpl>>, MockUserRepoImpl) {
        let mock_repo = MockUserRepoImpl::new();
        let service = Data::new(UserService::new(mock_repo.clone()));
        (service, mock_repo)
    }

    async fn parse_response_body<T: serde::de::DeserializeOwned>(response: HttpResponse) -> T {
        let body = to_bytes(response.into_body()).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[actix_web::test]
    async fn test_register_user_success() {
        let (service, mut mock_repo) = setup_mock_service();
        let controller = UserControllerImpl::new(service);
        
        let test_uuid = Uuid::now_v7();
        let request = RegisterRequest {
            username: Some("testuser".to_string()),
            public_key: "test_public_key".to_string(),
        };

        let expected_hash = sha256_hash(b"test_public_key").unwrap();
        let hash_clone = expected_hash.clone();

        mock_repo.expect_insert_user()
            .with(
                eq(&hash_clone),
                eq("test_public_key"),
                eq("testuser")
            )
            .times(1)
            .returning(move |_, _, _| Ok(test_uuid));

        let response = controller.register_user(Json(request)).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::CREATED);
        let body: RegisterResponse = parse_response_body(response).await;
        assert_eq!(body.user_id, test_uuid);
        assert_eq!(body.username, "testuser");
    }

    #[actix_web::test]
    async fn test_register_user_without_username() {
        let (service, mut mock_repo) = setup_mock_service();
        let controller = UserControllerImpl::new(service);
        
        let test_uuid = Uuid::now_v7();
        let request = RegisterRequest {
            username: None,
            public_key: "test_public_key".to_string(),
        };
        let expected_hash = sha256_hash(b"test_public_key").unwrap();
        let hash_clone = expected_hash.clone();

        mock_repo.expect_insert_user()
            .with(
                eq(&hash_clone),
                eq("test_public_key"),
                always(),
            )
            .times(1)
            .returning(move |_, _, _| Ok(test_uuid));

        let response = controller.register_user(Json(request)).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn test_get_user_success() {
        let (service, mut mock_repo) = setup_mock_service();
        let controller = UserControllerImpl::new(service);
        
        let test_uuid = Uuid::now_v7();
        let test_user = User {
            id: test_uuid,
            username: "testuser".to_string(),
            public_key: "test_public_key".to_string(),
            public_key_hash: sha256_hash(b"test_public_key").unwrap(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            last_login: Some(Utc::now().naive_utc()),
        };

        mock_repo.expect_get_user_by_id()
            .with(eq(test_uuid))
            .times(1)
            .returning(move |_| Ok(test_user.clone()));

        let response = controller.get_user(Path::from(test_uuid)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body: User = parse_response_body(response).await;
        assert_eq!(body.id, test_uuid);
        assert_eq!(body.username, "testuser");
    }

    #[actix_web::test]
    async fn test_get_user_not_found() {
        let (service, mut mock_repo) = setup_mock_service();
        let controller = UserControllerImpl::new(service);
        
        let test_uuid = Uuid::now_v7();

        mock_repo.expect_get_user_by_id()
            .with(eq(test_uuid))
            .times(1)
            .returning(|_| Err(AppError::NotFound("User not found".to_string())));

        let response = controller.get_user(Path::from(test_uuid)).await;
        assert!(response.is_err());
        
        if let Err(err) = response {
            assert_eq!(err.error_response().status(), StatusCode::NOT_FOUND);
        }
    }

    #[actix_web::test]
    async fn test_get_users_success() {
        let (service, mut mock_repo) = setup_mock_service();
        let controller = UserControllerImpl::new(service);
        
        let test_users = vec![
            User {
                id: Uuid::now_v7(),
                username: "user1".to_string(),
                public_key: "key1".to_string(),
                public_key_hash: sha256_hash(b"key1").unwrap(),
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
                last_login: Some(Utc::now().naive_utc())
            },
            User {
                id: Uuid::now_v7(),
                username: "user2".to_string(),
                public_key: "key2".to_string(),
                public_key_hash: sha256_hash(b"key2").unwrap(),
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
                last_login: Some(Utc::now().naive_utc())
            },
        ];

        // Test with limit
        mock_repo.expect_get_users()
            .with(eq(Some(10)))
            .times(1)
            .returning(move |_| Ok(test_users.clone()));

        let response = controller.get_users(Query::from(actix_web::web::Query(Some(10)))).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body: Vec<User> = parse_response_body(response).await;
        assert_eq!(body.len(), 2);

        // Test without limit
        mock_repo.expect_get_users()
            .with(eq(None))
            .times(1)
            .returning(move |_| Ok(test_users.clone()));

        let response = controller.get_users(Query::from(actix_web::web::Query(None))).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_update_user_success() {
        let (service, mut mock_repo) = setup_mock_service();
        let controller = UserControllerImpl::new(service);
        
        let test_uuid = Uuid::now_v7();
        let request = UpdateUserRequest {
            new_username: Some("newusername".to_string()),
            new_public_key: None,
        };

        mock_repo.expect_update_user()
            .with(
                eq(test_uuid),
                eq(Some("newusername".to_string())),
                eq(None),
                eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let response = controller.update_user(
            Path::from(test_uuid),
            Json(request),
        ).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_update_user_with_public_key() {
        let (service, mut mock_repo) = setup_mock_service();
        let controller = UserControllerImpl::new(service);
        
        let test_uuid = Uuid::now_v7();
        let new_key = "new_public_key".to_string();
        let new_key_hash = sha256_hash(&new_key.clone().into_bytes()).unwrap();
        let request = UpdateUserRequest {
            new_username: None,
            new_public_key: Some(new_key.clone()),
        };

        mock_repo.expect_update_user()
            .with(
                eq(test_uuid),
                eq(None),
                eq(Some(new_key)),
                eq(Some(new_key_hash)),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let response = controller.update_user(
            Path::from(test_uuid),
            Json(request),
        ).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }

    //error cases (validation errors, not found,...)
}
