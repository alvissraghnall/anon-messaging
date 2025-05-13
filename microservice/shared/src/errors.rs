use actix_web::{HttpResponse, ResponseError};
use sqlx::error::DatabaseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(sqlx::Error),

    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),

    #[error("Foreign key constraint violation: {0}")]
    ForeignKeyViolation(String),

    #[error("Data too long for column")]
    DataTooLong,

    #[error("Invalid input syntax")]
    InvalidInputSyntax,

    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Authentication failed")]
    AuthenticationError,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Public key hash error: {0}")]
    PublicKeyHashError(#[from] db::public_key_hash::PublicKeyHashError),

    #[error("Public key error: {0}")]
    PublicKeyError(#[from] db::public_key::PublicKeyError),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::DatabaseError(e) => {
                if let Some(db_err) = e.as_database_error() {
                    match db_err.kind() {
                        sqlx::error::ErrorKind::UniqueViolation => {
                            HttpResponse::Conflict().json(db_err.message())
                        }
                        sqlx::error::ErrorKind::ForeignKeyViolation => {
                            HttpResponse::BadRequest().json(db_err.message())
                        }
                        _ => HttpResponse::InternalServerError().json("Database error"),
                    }
                } else {
                    match e {
                        sqlx::Error::RowNotFound => {
                            HttpResponse::NotFound().json("Record not found")
                        }
                        _ => HttpResponse::InternalServerError().json("Database error"),
                    }
                }
            }
            AppError::UniqueViolation(msg) => HttpResponse::Conflict().json(msg),
            AppError::ForeignKeyViolation(msg) => HttpResponse::BadRequest().json(msg),
            AppError::DataTooLong => HttpResponse::BadRequest().json("Data too long for column"),
            AppError::InvalidInputSyntax => HttpResponse::BadRequest().json("Invalid input syntax"),
            AppError::ValidationError(e) => HttpResponse::BadRequest().json(e),
            AppError::NotFound(msg) => HttpResponse::NotFound().json(msg),
            AppError::AuthenticationError => {
                HttpResponse::Unauthorized().json("Authentication failed")
            }
            AppError::Forbidden(msg) => HttpResponse::Forbidden().json(msg),
            AppError::PublicKeyHashError(e) => HttpResponse::BadRequest().json(e.to_string()),
            AppError::PublicKeyError(e) => HttpResponse::BadRequest().json(e.to_string()),
            AppError::InternalError(msg) => HttpResponse::InternalServerError().json(msg),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let Some(db_err) = err.as_database_error() {
            match db_err.kind() {
                sqlx::error::ErrorKind::UniqueViolation => {
                    AppError::UniqueViolation(db_err.message().to_string())
                }
                sqlx::error::ErrorKind::ForeignKeyViolation => {
                    AppError::ForeignKeyViolation(db_err.message().to_string())
                }
                _ => AppError::DatabaseError(err),
            }
        } else {
            match err {
                sqlx::Error::RowNotFound => AppError::NotFound("Record not found".to_string()),
                _ => AppError::DatabaseError(err),
            }
        }
    }
}
