use base64::{
    alphabet,
    engine::{self, general_purpose, GeneralPurpose},
    Engine as _,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::sqlite::SqliteArgumentValue;
use sqlx::Type;
use sqlx::{sqlite::SqliteRow, Error as SqlxError, FromRow, Row};
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use utoipa::{PartialSchema, ToSchema};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublicKeyHashError {
    #[error("Invalid base64 encoding")]
    InvalidBase64,

    #[error("Decoded hash is not 32 bytes (not a valid SHA-256 hash)")]
    InvalidHashLength,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ToSchema)]
pub struct PublicKeyHash(String);

const CUSTOM_ENGINE: GeneralPurpose =
    GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

impl PublicKeyHash {
    pub fn new(hash: String) -> Result<Self, PublicKeyHashError> {
        match CUSTOM_ENGINE.decode(&hash) {
            Ok(bytes) => {
                if bytes.len() == 32 {
                    Ok(PublicKeyHash(hash))
                } else {
                    Err(PublicKeyHashError::InvalidHashLength)
                }
            }
            Err(_) => Err(PublicKeyHashError::InvalidBase64),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'r> sqlx::FromRow<'r, SqliteRow> for PublicKeyHash {
    fn from_row(row: &SqliteRow) -> Result<Self, SqlxError> {
        let s: String = row.try_get("public_key")?;
        PublicKeyHash::new(s).map_err(|e| SqlxError::Decode(Box::new(e)))
    }
}

impl fmt::Display for PublicKeyHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for PublicKeyHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PublicKeyHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(PublicKeyHash(s))
    }
}

impl Type<sqlx::Sqlite> for PublicKeyHash {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for PublicKeyHash {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        Ok(PublicKeyHash(s))
    }
}

impl sqlx::Encode<'_, sqlx::Sqlite> for PublicKeyHash {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<SqliteArgumentValue<'_>>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn Error + Send + Sync>> {
        <String as sqlx::Encode<'_, sqlx::Sqlite>>::encode_by_ref(&self.0, buf)
    }
}
