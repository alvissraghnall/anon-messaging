use std::sync::Arc;
use sqlx::{Pool, Sqlite, SqlitePool};
use db::models::User;
use db::uuid::Uuid;
use shared::models::RegisterRequest;
use shared::errors::AppError;
use service::user::UserRepository;
use db::{public_key::PublicKey, public_key_hash::PublicKeyHash};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use db::faker_rand::en_us::names::FullName;
use service::p256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use service::rand::{self, Rng};


static CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

pub async fn create_test_connection_pool() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    
    sqlx::migrate!("../db/migrations")
        .run(&pool)
        .await?;
    
    Ok(pool)
}

pub async fn create_test_users(pool: &SqlitePool, count: usize) -> Result<Vec<Uuid>, AppError> {
    let mut users = Vec::with_capacity(count);

    let user_repo: Arc<dyn UserRepository> = Arc::new(pool.clone());
    
    for i in 0..count {

        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);

        let b64_key = CUSTOM_ENGINE.encode(verifying_key.to_encoded_point(true).as_bytes());
        let public_key = PublicKey::new(b64_key).unwrap();
        
        let user_id = user_repo
        .insert_user(public_key.as_str(), &format!("testuser{}", i))
        .await
        .unwrap();
        
        users.push(user_id);
    }
    
    Ok(users)
}

pub async fn create_test_user_with_id(
    pool: &SqlitePool,
    user_id: Uuid,
) -> Result<(), AppError> {
    let full_name = rand::random::<FullName>().to_string();

    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);

    let b64_key = CUSTOM_ENGINE.encode(verifying_key.to_encoded_point(true).as_bytes());
    let public_key = PublicKey::new(b64_key).unwrap();

    let public_key_hash = public_key.to_hash().unwrap();

    println!("{}", 54);
    
    let user = sqlx::query(
        r#"INSERT INTO users (id, public_key, public_key_hash, username)
         VALUES ($1, $2, $3, $4)
         "#,
    )
    .bind(user_id)
    .bind(public_key)
    .bind(public_key_hash)
    .bind(full_name)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_test_user_from_request(
    pool: &SqlitePool,
    request: &RegisterRequest
) -> Result<User, AppError> {
    let user_id = Uuid::now_v7();
    let full_name = rand::random::<FullName>().to_string();
    
    let b64_key = CUSTOM_ENGINE.encode(request.public_key.clone());
    let public_key = PublicKey::new(b64_key).unwrap();

    let public_key_hash = public_key.to_hash().unwrap();

    let user = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (id, public_key, public_key_hash, username)
         VALUES ($1, $2, $3, $4)
         "#,
    )
    .bind(user_id)
    .bind(public_key)
    .bind(public_key_hash)
    .bind(full_name)
    .fetch_one(pool)
    .await?;
    
    Ok(user)
}
