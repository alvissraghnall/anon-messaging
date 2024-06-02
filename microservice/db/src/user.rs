use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
     pub id: i64,
     pub user_id: String,
     pub public_key_hash: String,
}
