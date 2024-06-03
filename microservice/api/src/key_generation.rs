use db::db::{create_db_pool, insert_user_with_retry};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use rand::Rng;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyGenerationRequest {
    pub custom_user_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyGenerationResponse {
    pub user_id: String,
    pub encrypted_private_key: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Option<i64>,
    pub user_id: String,
    pub public_key_hash: String,
}

/// Generate a new key pair and store the user in the database
async fn generate_keys(
    pool: web::Data<SqlitePool>,
    request: web::Json<KeyGenerationRequest>,
) -> impl Responder {
    // Generate a new key pair
    let key_pair = KeyPair::generate();
    let public_key_hash = key_pair.public_key_hash();

    // Derive or validate the user_id
    let user_id = match &request.custom_user_id {
        Some(id) => {
            if let Err(e) = validate_user_id(id) {
                return HttpResponse::BadRequest().body(e);
            }
            id.clone()
        }
        None => generate_user_id(),
    };

    // Store the user in the database
    if let Err(e) = insert_user_with_retry(&pool, &user_id, &public_key_hash).await {
        return HttpResponse::InternalServerError().body(format!("Failed to insert user: {}", e));
    }

    let encrypted_private_key = base64::encode(&key_pair.private_key);

    // Return the response
    HttpResponse::Ok().json(KeyGenerationResponse {
        user_id,
        encrypted_private_key,
    })
}

/// Validate a custom user_id
fn validate_user_id(user_id: &str) -> Result<(), String> {
    if user_id.len() > 20 {
        return Err("user_id must be 20 characters or less".to_string());
    }
    if !user_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("user_id can only contain alphanumeric characters and underscores".to_string());
    }
    Ok(())
}

/// Generate a unique 8-character hex string
fn generate_user_id() -> String {
    let uuid = Uuid::new_v4();
    let uuid_hex = uuid.as_simple().to_string();
    uuid_hex[..8].to_string()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create a database connection pool
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create database pool");

    // Start the Actix Web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))  // Share the database pool
            .route("/generate-keys", web::post().to(generate_keys))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
