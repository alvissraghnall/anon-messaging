use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Base64 encoding error")]
    Base64Error,
    #[error("Hashing error")]
    HashError,
    #[error("Invalid key format")]
    InvalidKeyFormat,
    #[error("Signature verification failed")]
    VerificationFailed,
}

pub fn sha256_hash(data: &[u8]) -> Result<String, CryptoError> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    Ok(general_purpose::STANDARD.encode(result))
}

pub fn base64_encode(data: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

pub fn base64_decode(data: &str) -> Result<Vec<u8>, CryptoError> {
    general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .map_err(|_| CryptoError::Base64Error)
}

pub fn normalize_for_signing(input: &str) -> String {
    input.trim().to_lowercase()
}

pub fn format_signing_payload(method: &str, path: &str, timestamp: i64, body_hash: &str) -> String {
    format!(
        "{}\n{}\n{}\n{}",
        normalize_for_signing(method),
        normalize_for_signing(path),
        timestamp,
        body_hash
    )
}

pub fn validate_signature_format(signature: &str) -> Result<(), CryptoError> {
    if signature.len() != 344 {
        // Expected length for 2048-bit RSA signature
        return Err(CryptoError::InvalidKeyFormat);
    }
    base64_decode(signature)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash() {
        let hash = sha256_hash(b"test").unwrap();
        assert_eq!(hash, "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg=");
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"test data";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_signature_format_validation() {
        let bad_sig = "invalid!";
        assert!(validate_signature_format(bad_sig).is_err());
    }
}
