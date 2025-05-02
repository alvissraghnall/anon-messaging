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
