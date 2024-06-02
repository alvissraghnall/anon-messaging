use sqlx::SqlitePool;
use dotenv::dotenv;
use std::env;

pub async fn create_db_pool() -> Result<SqlitePool, sqlx::Error> {
	dotenv().ok(); 
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	SqlitePool::connect(&database_url).await
}
