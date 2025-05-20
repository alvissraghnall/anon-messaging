use crate::public_key_hash::{PublicKeyHash, PublicKeyHashError};
use base64::{
    alphabet,
    engine::{self, general_purpose, GeneralPurpose},
    Engine as _,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::{
    sqlite::{SqliteArgumentValue, SqliteRow},
    Error as SqlxError, FromRow, Row, Type,
};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Error, Debug)]
pub enum PublicKeyError {
    #[error("Invalid base64 encoding for public key")]
    InvalidBase64,
}

const CUSTOM_ENGINE: GeneralPurpose =
    GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

#[derive(Debug, Clone, PartialEq, Eq, Hash, ToSchema)]
pub struct PublicKey(String);

impl PublicKey {
    pub fn new(key: String) -> Result<Self, PublicKeyError> {
        match CUSTOM_ENGINE.decode(&key) {
            Ok(_) => Ok(PublicKey(key)),
            Err(_) => Err(PublicKeyError::InvalidBase64),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_hash(&self) -> Result<PublicKeyHash, PublicKeyHashError> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.0.as_bytes());
        let result = hasher.finalize();
        let hash = CUSTOM_ENGINE.encode(result);
        Ok(PublicKeyHash::new(hash)?)
    }
}

impl TryFrom<String> for PublicKey {
    type Error = PublicKeyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(PublicKey::new(value)?)
    }
}

impl FromStr for PublicKey {
    type Err = PublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PublicKey::new(s.to_string())?)
    }
}

impl<'r> sqlx::FromRow<'r, SqliteRow> for PublicKey {
    fn from_row(row: &SqliteRow) -> Result<Self, SqlxError> {
        let s: String = row.try_get("public_key")?;
        PublicKey::new(s).map_err(|e| SqlxError::Decode(e.into()))
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PublicKey::new(s).map_err(serde::de::Error::custom)
    }
}

impl Type<sqlx::Sqlite> for PublicKey {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for PublicKey {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        Ok(PublicKey(s))
    }
}

impl sqlx::Encode<'_, sqlx::Sqlite> for PublicKey {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<SqliteArgumentValue<'_>>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn Error + Send + Sync>> {
        <String as sqlx::Encode<'_, sqlx::Sqlite>>::encode_by_ref(&self.0, buf)
    }
}
