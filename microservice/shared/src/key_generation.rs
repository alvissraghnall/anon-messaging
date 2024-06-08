use std::hash::Hash;

use aes_gcm::{
    aead::{
        generic_array::{sequence::GenericSequence, GenericArray},
        Aead, Nonce,
    },
    AeadCore, Aes256Gcm, KeyInit,
};
use db::db::{create_db_pool, insert_user};
use p256::ecdsa::{SigningKey, VerifyingKey};
//use rand::rngs::OsRng;
use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Params, Pbkdf2,
};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
pub struct KeyPair {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
}

impl KeyPair {
    /// Generates a new ECC key pair.
    pub fn generate() -> Self {
        let mut rng = OsRng;
        let signing_key = SigningKey::random(&mut rng); // Private key
        let verifying_key = VerifyingKey::from(&signing_key); // Public key

        KeyPair {
            private_key: signing_key.to_bytes().to_vec(),
            public_key: verifying_key.to_sec1_bytes().to_vec(),
        }
    }

    /// Returns the private key as a byte vector.
    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }

    /// Returns the public key as a byte vector.
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }
    /// Serializes the key pair to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Deserializes the key pair from a JSON string.
    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap()
    }

    /// Returns the SHA-256 hash of the public key as a hex string.
    pub fn public_key_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.public_key);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    pub fn encrypt_private_key(
        &self,
        passphrase: &str,
    ) -> Result<(Vec<u8>, Vec<u8>, String), String> {
        let salt = SaltString::generate(&mut OsRng);
        let salt_str = salt.to_string();

        let params = Params {
            rounds: 100_000,
            output_length: 32,
        };

        let key = Pbkdf2
            .hash_password_customized(passphrase.as_bytes(), None, None, params, &salt)
            .map_err(|e| e.to_string())?
            .hash
            .ok_or("Failed to derive key")?;

        let cipher = Aes256Gcm::new_from_slice(&key.as_bytes()).map_err(|e| e.to_string())?;

        //let mut rng = OsRng;
        //let nonce = Nonce::generate(|_| rng.next_u64().try_into().unwrap());
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let encrypted_key = cipher
            .encrypt(&nonce, self.private_key.as_ref())
            .map_err(|e| e.to_string())?;

        Ok((encrypted_key, nonce.to_vec(), salt_str))
    }

    /// Decrypts the private key using a user-provided passphrase.
    pub fn decrypt_private_key(
        encrypted_key: &[u8],
        nonce: &[u8],
        passphrase: &str,
        salt_str: &str,
    ) -> Result<Vec<u8>, String> {
        // Derive the key using PBKDF2
        let salt = SaltString::from_b64(salt_str).map_err(|e| e.to_string())?;
        let params = Params {
            rounds: 100_000,
            output_length: 32,
        };
        let key = Pbkdf2
            .hash_password_customized(passphrase.as_bytes(), None, None, params, &salt)
            .map_err(|e| e.to_string())?
            .hash
            .ok_or("Failed to derive key")?;

        // Initialize AES-256-GCM
        let cipher = Aes256Gcm::new_from_slice(&key.as_bytes()).map_err(|e| e.to_string())?;

        // Decrypt the private key
        let decrypted_key = cipher
            .decrypt(&GenericArray::from_slice(nonce), encrypted_key)
            .map_err(|e| e.to_string())?;

        Ok(decrypted_key)
    }
}

#[cfg(test)]
#[path = "key_generation.test.rs"]
mod tests;
