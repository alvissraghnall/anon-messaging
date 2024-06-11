use shared::data_encryption::encrypt_at_rest;
use sqlx::{SqlitePool};
use shared::key_generation::KeyPair;
use db::db::{create_db_pool, insert_user_with_retry, generate_user_id, User};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyGenerationRequest {
    pub custom_user_id: Option<String>,
    pub keyphrase: String, 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyGenerationResponse {
    pub user_id: String,
    pub encrypted_private_key: String,
    pub encryption_salt: String, 
    pub encryption_nonce: String, 
}

pub async fn generate_keys(
    pool: web::Data<SqlitePool>,
    request: web::Json<KeyGenerationRequest>,
) -> impl Responder {
	
	if let Err(e) = validate_keyphrase(&request.keyphrase) {
        return HttpResponse::BadRequest().body(e);
    }

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

    // Encrypt the private key using the keyphrase
    let (encrypted_private_key, nonce, salt) = match key_pair.encrypt_private_key(&request.keyphrase) {
        Ok(result) => result,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to encrypt private key: {}", e)),
    };

    // Encrypt the nonce and salt before storing them
    let server_key = env::var("SERVER_KEY").expect("SERVER_KEY must be set");
	//dbg!("{}", &server_key);
	//println!("{:?}", server_key.as_bytes());
	//println!("{:?}", std::env::vars());

    let encrypted_nonce = encrypt_at_rest(&nonce, server_key.as_bytes());

	let salt_bytes = salt.as_bytes();
	let encrypted_salt = encrypt_at_rest(&salt_bytes, server_key.as_bytes());

	let encoded_salt = base64::encode(encrypted_salt);
	let encoded_nonce = base64::encode(encrypted_nonce);
	let encoded_private_key = base64::encode(encrypted_private_key);

	// store user in database
	insert_user_with_retry(
	    &pool,
	    &user_id,
	    &public_key_hash,
	    &encoded_private_key,
	    &encoded_nonce,
	    &encoded_salt,
	)
	.await
	.map(|final_user_id| {
	    // Handle the success case
	    println!("User inserted successfully: {}", final_user_id);
		HttpResponse::Ok().json(KeyGenerationResponse {
	        user_id: final_user_id,
	        encrypted_private_key: encoded_private_key,
	        encryption_nonce: encoded_nonce,
	        encryption_salt: encoded_salt,
	    })
	})
	.map_err(|e| {
	    eprintln!("Failed to insert user: {:?}", e);
	    HttpResponse::InternalServerError().body(format!("Failed to insert user: {}", e))
	})
	.unwrap_or_else(|e| {
        HttpResponse::InternalServerError().body(format!("Failed to insert user."))
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

fn validate_keyphrase(keyphrase: &str) -> Result<(), String> {
    if keyphrase.len() < 8 {
        return Err("Keyphrase must be at least 8 characters long".to_string());
    }

    let has_uppercase = keyphrase.chars().any(|c| c.is_ascii_uppercase());
    let has_lowercase = keyphrase.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = keyphrase.chars().any(|c| c.is_ascii_digit());
    // let has_special = keyphrase.chars().any(|c| !c.is_ascii_alphanumeric());

    if !(has_uppercase && has_lowercase && has_digit) {
        return Err("Keyphrase must include uppercase, lowercase, digits, and special characters".to_string());
    }

    Ok(())
}


#[cfg(test)]
#[path = "key_generation.test.rs"]
mod tests;
