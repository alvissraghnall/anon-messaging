use serde::{Deserialize, Serialize};
use validator::Validate;
use db::{uuid::{self, Uuid}};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,

    #[validate(length(min = 50, message = "Invalid public key format"))]
    pub public_key: String, // Base64-encoded SPKI format
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    #[serde(with = "uuid::serde::simple")]
    pub user_id: Uuid,
    pub username: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub new_username: Option<String>,

    pub new_public_key: Option<String>, // Base64-encoded SPKI formaat
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SignedRequest<T> {
    pub payload: T,
    pub signature: String,
    pub timestamp: i64,
    pub public_key: Option<String>, // Only needed for first request
}
