use super::*;
use std::collections::HashSet;
use ark_std::Zero;

#[test]
fn test_generate_ephemeral_keypair() {
    let UserB { esk, epub } = UserB::generate_ephemeral_keypair();

    // Ensure the secret key is not zero
    assert!(!esk.is_zero(), "Secret key should not be zero");

    // Ensure the public key is correctly derived
    let generator = Ed25519::generator();
    let expected_pub = generator.mul(esk);
    assert_eq!(epub, expected_pub, "Public key must be esk * G");
}

#[test]
fn test_generate_unique_keys() {
    let mut esk_set = HashSet::new();
    let mut epub_set = HashSet::new();

    for _ in 0..6 {
        let UserB { esk, epub } = UserB::generate_ephemeral_keypair();
        assert!(esk_set.insert(esk));
        assert!(epub_set.insert(epub));
    }
}

#[test]
fn test_public_key_on_curve() {
    let UserB { epub, .. } = UserB::generate_ephemeral_keypair();

    // Check that epub is a valid Ed25519 point
    let affine_epub = epub.into_affine();

    assert!(affine_epub.is_on_curve());
    assert!(affine_epub.is_in_correct_subgroup_assuming_on_curve());
}

#[test]
fn test_compute_shared_secret() {
    // Generate a random ephemeral secret key
    let esk = Fr::rand(&mut rand::thread_rng());

    // Generate a random public key for User A
    let pk_a = Ed25519::generator().mul(&esk);

    // Create a new instance of the struct
    let instance = UserB { esk, epub: Ed25519::generator().mul(&esk) };

    // Compute the shared secret using the instance
    let shared_secret = instance.compute_shared_secret(&pk_a);

    // Verify that the shared secret is correct
    assert_eq!(shared_secret, pk_a.mul(&instance.esk));
}

#[test]
fn test_compute_shared_secret_with_invalid_input() {
    // Generate a random ephemeral secret key
    let esk = Fr::rand(&mut rand::thread_rng());

    // Create a new instance of the struct
    let instance = UserB { esk, epub: Ed25519::generator().mul(&esk) };

    // Create an invalid public key for User A: the zero point
    let pk_a = Ed25519::zero();

    // Compute the shared secret using the instance
    let shared_secret = instance.compute_shared_secret(&pk_a);

    // Verify that the shared secret is the zero point
    assert_eq!(shared_secret, Ed25519::zero());
}

#[test]
fn test_compute_shared_secret_with_multiple_inputs() {
    // Generate a random ephemeral secret key
    let esk = Fr::rand(&mut rand::thread_rng());

    // Create a new instance of the struct
    let instance = UserB { esk, epub: Ed25519::generator().mul(&esk) };

    // Generate multiple random public keys for User A
    let pks_a = vec![
        Ed25519::generator().mul(&esk),
        Ed25519::generator().mul(&Fr::rand(&mut rand::thread_rng())),
        Ed25519::generator().mul(&Fr::rand(&mut rand::thread_rng())),
    ];

    // Compute the shared secrets using the instance
    let shared_secrets = pks_a.iter().map(|pk_a| instance.compute_shared_secret(pk_a)).collect::<Vec<_>>();

    // Verify that the shared secrets are correct
    for (i, shared_secret) in shared_secrets.iter().enumerate() {
        assert_eq!(*shared_secret, pks_a[i].mul(&instance.esk));
    }
}

#[test]
fn test_derived_keys_length() {
    let user_b = UserB::generate_ephemeral_keypair();
    let sk_a = Fr::rand(&mut thread_rng());
    let pk_a = Ed25519::generator().mul(sk_a);
    let shared_secret = user_b.compute_shared_secret(&pk_a);
    let derived_keys = user_b.derive_keys(&shared_secret);

    // Check that derived keys have correct lengths
    assert_eq!(derived_keys.randomness.len(), 32, "Randomness should be 32 bytes");
    assert_eq!(derived_keys.enc_key.len(), 32, "Encryption key should be 32 bytes");
}

#[test]
fn test_derived_keys_deterministic() {
    // Generate two sets of keys with same inputs
    let user_b = UserB::generate_ephemeral_keypair();
    let sk_a = Fr::rand(&mut thread_rng());
    let pk_a = Ed25519::generator().mul(sk_a);
    let shared_secret = user_b.compute_shared_secret(&pk_a);
    
    let derived_keys_1 = user_b.derive_keys(&shared_secret);
    let derived_keys_2 = user_b.derive_keys(&shared_secret);

    // Same inputs should produce same outputs
    assert_eq!(
        derived_keys_1.randomness,
        derived_keys_2.randomness,
        "Randomness derivation should be deterministic"
    );
    assert_eq!(
        derived_keys_1.enc_key,
        derived_keys_2.enc_key,
        "Encryption key derivation should be deterministic"
    );
}

#[test]
fn test_derived_keys_different_shared_secrets() {
    let user_b = UserB::generate_ephemeral_keypair();
    
    // Generate two different shared secrets
    let sk_a1 = Fr::rand(&mut thread_rng());
    let sk_a2 = Fr::rand(&mut thread_rng());
    let pk_a1 = Ed25519::generator().mul(sk_a1);
    let pk_a2 = Ed25519::generator().mul(sk_a2);
    
    let shared_secret1 = user_b.compute_shared_secret(&pk_a1);
    let shared_secret2 = user_b.compute_shared_secret(&pk_a2);
    
    let derived_keys_1 = user_b.derive_keys(&shared_secret1);
    let derived_keys_2 = user_b.derive_keys(&shared_secret2);

    // Different inputs should produce different outputs
    assert_ne!(
        derived_keys_1.randomness,
        derived_keys_2.randomness,
        "Different shared secrets should produce different randomness"
    );
    assert_ne!(
        derived_keys_1.enc_key,
        derived_keys_2.enc_key,
        "Different shared secrets should produce different encryption keys"
    );
}

#[test]
fn test_derived_keys_distinctness() {
    let user_b = UserB::generate_ephemeral_keypair();
    let sk_a = Fr::rand(&mut thread_rng());
    let pk_a = Ed25519::generator().mul(sk_a);
    let shared_secret = user_b.compute_shared_secret(&pk_a);
    let derived_keys = user_b.derive_keys(&shared_secret);

    // Randomness and encryption key should be different
    assert_ne!(
        derived_keys.randomness,
        derived_keys.enc_key,
        "Randomness and encryption key should be distinct"
    );
}

#[test]
fn test_derived_keys_non_zero() {
    let user_b = UserB::generate_ephemeral_keypair();
    let sk_a = Fr::rand(&mut thread_rng());
    let pk_a = Ed25519::generator().mul(sk_a);
    let shared_secret = user_b.compute_shared_secret(&pk_a);
    let derived_keys = user_b.derive_keys(&shared_secret);

    // Check that derived keys are not all zeros
    assert_ne!(
        derived_keys.randomness,
        [0u8; 32],
        "Randomness should not be all zeros"
    );
    assert_ne!(
        derived_keys.enc_key,
        [0u8; 32],
        "Encryption key should not be all zeros"
    );
}

#[test]
fn test_key_derivation_with_identity_point() {
    let user_b = UserB::generate_ephemeral_keypair();
    let identity = Ed25519::generator();
    let derived_keys = user_b.derive_keys(&identity);

    // Even with identity point, should still produce valid keys
    assert_ne!(
        derived_keys.randomness,
        [0u8; 32],
        "Randomness should be non-zero even with identity point"
    );
    assert_ne!(
        derived_keys.enc_key,
        [0u8; 32],
        "Encryption key should be non-zero even with identity point"
    );
}

#[test]
fn test_serialize_deserialize_consistency() {
    let user_b = UserB::generate_ephemeral_keypair();
    let sk_a = Fr::rand(&mut thread_rng());
    let pk_a = Ed25519::generator().mul(sk_a);
    let shared_secret = user_b.compute_shared_secret(&pk_a);
    
    // Serialize and deserialize the shared secret
    let mut serialized = Vec::new();
    shared_secret.serialize_uncompressed(&mut serialized).unwrap();
    
    // Verify serialization is consistent
    let mut serialized2 = Vec::new();
    shared_secret.serialize_uncompressed(&mut serialized2).unwrap();
    assert_eq!(
        serialized,
        serialized2,
        "Serialization should be deterministic"
    );
}

#[test]
fn test_domain_separation() {
    let user_b = UserB::generate_ephemeral_keypair();
    let sk_a = Fr::rand(&mut thread_rng());
    let pk_a = Ed25519::generator().mul(sk_a);
    let shared_secret = user_b.compute_shared_secret(&pk_a);
    
    // Manual derivation with different domain tags
    let mut ss_bytes = Vec::new();
    shared_secret.serialize_uncompressed(&mut ss_bytes).unwrap();
    
    let mut hasher1 = Sha256::new();
    hasher1.update(b"test_domain_1");
    hasher1.update(&ss_bytes);
    let key1 = hasher1.finalize();
    
    let mut hasher2 = Sha256::new();
    hasher2.update(b"test_domain_2");
    hasher2.update(&ss_bytes);
    let key2 = hasher2.finalize();
    
    assert_ne!(
        key1,
        key2,
        "Different domain separation tags should produce different outputs"
    );
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let user_b = UserB::generate_ephemeral_keypair();
    let sk_a = Fr::rand(&mut thread_rng());
    let pk_a = Ed25519::generator().mul(sk_a);
    let shared_secret = user_b.compute_shared_secret(&pk_a);
    let derived_keys = user_b.derive_keys(&shared_secret);
    
    let message = b"Hello, World!";
    
    // Encrypt
    let ciphertext = user_b.encrypt_message(message, &derived_keys.enc_key)
        .expect("Encryption should succeed");
        
    // Decrypt
    let decrypted = user_b.decrypt_message(&ciphertext, &derived_keys.enc_key)
        .expect("Decryption should succeed");
        
    assert_eq!(message, decrypted.as_slice());
}

#[test]
fn test_different_messages_produce_different_ciphertexts() {
    let user_b = UserB::generate_ephemeral_keypair();
    let derived_keys = DerivedKeys {
        randomness: [0u8; 32],
        enc_key: [1u8; 32],
    };
    
    let message1 = b"Hello";
    let message2 = b"World";
    
    let ciphertext1 = user_b.encrypt_message(message1, &derived_keys.enc_key)
        .expect("Encryption should succeed");
    let ciphertext2 = user_b.encrypt_message(message2, &derived_keys.enc_key)
        .expect("Encryption should succeed");
        
    assert_ne!(ciphertext1.encrypted, ciphertext2.encrypted);
}

#[test]
fn test_same_message_different_nonces() {
    let user_b = UserB::generate_ephemeral_keypair();
    let derived_keys = DerivedKeys {
        randomness: [0u8; 32],
        enc_key: [1u8; 32],
    };
    
    let message = b"Same message";
    
    let ciphertext1 = user_b.encrypt_message(message, &derived_keys.enc_key)
        .expect("Encryption should succeed");
    let ciphertext2 = user_b.encrypt_message(message, &derived_keys.enc_key)
        .expect("Encryption should succeed");
        
    assert_ne!(ciphertext1.encrypted, ciphertext2.encrypted);
    assert_ne!(ciphertext1.nonce, ciphertext2.nonce);
}

#[test]
fn test_wrong_key_fails_decryption() {
    let user_b = UserB::generate_ephemeral_keypair();
    let correct_key = [1u8; 32];
    let wrong_key = [2u8; 32];
    
    let message = b"Secret message";
    
    let ciphertext = user_b.encrypt_message(message, &correct_key)
        .expect("Encryption should succeed");
        
    let decryption_result = user_b.decrypt_message(&ciphertext, &wrong_key);
    assert!(decryption_result.is_err());
}

#[test]
fn test_modified_ciphertext_fails_decryption() {
    let user_b = UserB::generate_ephemeral_keypair();
    let key = [1u8; 32];
    let message = b"Secret message";
    
    let mut ciphertext = user_b.encrypt_message(message, &key)
        .expect("Encryption should succeed");
        
    // Modify the ciphertext
    if let Some(byte) = ciphertext.encrypted.get_mut(0) {
        *byte ^= 1;
    }
    
    let decryption_result = user_b.decrypt_message(&ciphertext, &key);
    assert!(decryption_result.is_err());
}

#[test]
fn test_empty_message() {
    let user_b = UserB::generate_ephemeral_keypair();
    let key = [1u8; 32];
    let message = b"";
    
    let ciphertext = user_b.encrypt_message(message, &key)
        .expect("Encryption should succeed");
    let decrypted = user_b.decrypt_message(&ciphertext, &key)
        .expect("Decryption should succeed");
        
    assert_eq!(message, decrypted.as_slice());
}

#[test]
fn test_large_message() {
    let user_b = UserB::generate_ephemeral_keypair();
    let key = [1u8; 32];
    let message = vec![0u8; 1000000]; // 1MB of zeros
    
    let ciphertext = user_b.encrypt_message(&message, &key)
        .expect("Encryption should succeed");
    let decrypted = user_b.decrypt_message(&ciphertext, &key)
        .expect("Decryption should succeed");
        
    assert_eq!(message, decrypted);
}
