use base64::{
    alphabet,
    engine::{self, general_purpose, GeneralPurpose},
    Engine as _,
};
use db::uuid::{self, Uuid};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema, Clone, PartialEq)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,

    #[validate(length(min = 50, message = "Invalid public key format"))]
    pub public_key: String, // Base64-encoded SPKI format
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq)]
pub struct RegisterResponse {
    #[serde(with = "uuid::serde::simple")]
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema, PartialEq, Clone)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub new_username: Option<String>,

    #[validate(length(min = 50, message = "Invalid public key format"))]
    pub new_public_key: Option<String>, // Base64-encoded SPKI formaat
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SignedRequest<T> {
    pub payload: T,
    pub signature: String,
    pub timestamp: i64,
    pub public_key: Option<String>, // Only needed for first request
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct CreateMessageResponse {
    message_id: i64,
}

impl CreateMessageResponse {
    pub fn new(message_id: i64) -> Self {
        Self { message_id }
    }

    pub fn get_message_id(&self) -> i64 {
        self.message_id
    }
}

pub const CUSTOM_ENGINE: GeneralPurpose =
    GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

#[derive(ToSchema, Serialize, Deserialize, Debug, Validate, Clone)]
pub struct CreateMessageRequest {
    #[serde(with = "uuid::serde::simple")]
    pub sender_id: Uuid,

    #[serde(with = "uuid::serde::simple")]
    pub recipient_id: Uuid,

    #[validate(custom(function = "validate_base64_min_len_4"))]
    pub encrypted_content: String,

    #[validate(custom(function = "validate_optional_base64_max_512"))]
    pub signature: Option<String>,

    pub parent_id: Option<i64>,
}

fn validate_base64_min_len_4(val: &str) -> Result<(), ValidationError> {
    match CUSTOM_ENGINE.decode(val) {
        Ok(decoded) if decoded.len() >= 4 => Ok(()),
        Ok(_) => Err(ValidationError::new("encrypted_content_too_short")),
        Err(_) => Err(ValidationError::new("invalid_base64")),
    }
}

fn validate_optional_base64_max_512(val: &str) -> Result<(), ValidationError> {
    match CUSTOM_ENGINE.decode(val) {
        Ok(decoded) if decoded.len() <= 512 => Ok(()),
        Ok(_) => Err(ValidationError::new("signature_too_long")),
        Err(_) => Err(ValidationError::new("invalid_base64")),
    }
}
