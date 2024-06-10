use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce, AeadCore
};
use base64;

/// Encrypts data at rest using AES-256-GCM.
pub fn encrypt_at_rest(data: &[u8], server_key: &[u8]) -> Vec<u8> {
    // Create an AES-256-GCM cipher instance
    let cipher = Aes256Gcm::new_from_slice(server_key).expect("Invalid key length");

    // Generate a random nonce (12 bytes for AES-GCM)
	let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Encrypt the data
    let encrypted_data = cipher
        .encrypt(&nonce, data)
        .expect("Encryption failed");

    // Combine the nonce and encrypted data into a single vector
    let mut result = nonce.to_vec();
    result.extend(encrypted_data);
    result
}

/// Decrypts data at rest using AES-256-GCM.
pub fn decrypt_at_rest(encrypted_data: &[u8], server_key: &[u8]) -> Vec<u8> {
    // Create an AES-256-GCM cipher instance
    let cipher = Aes256Gcm::new_from_slice(server_key).expect("Invalid key length");

    // Split the nonce and encrypted data
    let nonce = Nonce::from_slice(&encrypted_data[..12]);
    let encrypted_data = &encrypted_data[12..];

    // Decrypt the data
    cipher
        .decrypt(nonce, encrypted_data)
        .expect("Decryption failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_encrypt_decrypt() {
        let server_key = [0u8; 32]; // 256-bit key
        let data = b"Hello, World!";
        let encrypted_data = encrypt_at_rest(data, &server_key);
        let decrypted_data = decrypt_at_rest(&encrypted_data, &server_key);
        assert_eq!(data, &decrypted_data[..]);
    }

    #[test]
    fn test_encrypt_decrypt_empty_data() {
        let server_key = [0u8; 32]; // 256-bit key
        let data = b"";
        let encrypted_data = encrypt_at_rest(data, &server_key);
        let decrypted_data = decrypt_at_rest(&encrypted_data, &server_key);
        assert_eq!(data, &decrypted_data[..]);
    }

    #[test]
    fn test_encrypt_decrypt_large_data() {
        let server_key = [0u8; 32]; // 256-bit key
        let mut data = Vec::new();
        let mut rng = rand::thread_rng();
        for _ in 0..1024 {
            data.push(rng.gen::<u8>());
        }
        let encrypted_data = encrypt_at_rest(&data, &server_key);
        let decrypted_data = decrypt_at_rest(&encrypted_data, &server_key);
        assert_eq!(data, decrypted_data);
    }

    #[test]
    #[should_panic]
    fn test_encrypt_invalid_key() {
        let server_key = [0u8; 31]; // invalid key length
        let data = b"Hello, World!";
        encrypt_at_rest(data, &server_key);
    }

    #[test]
    #[should_panic]
    fn test_decrypt_invalid_nonce() {
        let server_key = [0u8; 32]; // 256-bit key
        let data = b"Hello, World!";
        let encrypted_data = encrypt_at_rest(data, &server_key);
        let mut invalid_nonce = encrypted_data[..12].to_vec();
        invalid_nonce[0] ^= 1; // corrupt the nonce
        let invalid_encrypted_data = [invalid_nonce, encrypted_data[12..].to_vec()].concat();
        decrypt_at_rest(&invalid_encrypted_data, &server_key);
    }
}
