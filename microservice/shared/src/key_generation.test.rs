use generic_array::GenericArray;
use super::*;

#[test]
fn test_key_pair_generation() {
    let key_pair = KeyPair::generate();
    
    // Test that keys are not empty
    assert!(!key_pair.private_key.is_empty());
    assert!(!key_pair.public_key.is_empty());
    
    // Test that keys have correct lengths for P-256
    assert_eq!(key_pair.private_key.len(), 32); // P-256 private key is 32 bytes
    assert_eq!(key_pair.public_key.len(), 65);  // Public key length is 65 bytes.
}

#[test]
fn test_key_accessors() {
    let key_pair = KeyPair::generate();
    let private_key = key_pair.private_key();
    let public_key = key_pair.public_key();

    // Test that accessors return correct data
    assert_eq!(private_key, key_pair.private_key.as_slice());
    assert_eq!(public_key, key_pair.public_key.as_slice());
}

#[test]
fn test_serialization_deserialization() {
    let original_key_pair = KeyPair::generate();
    let json = original_key_pair.to_json();
    let deserialized_key_pair = KeyPair::from_json(&json);

    // Test that serialization/deserialization preserves the keys
    assert_eq!(original_key_pair.private_key, deserialized_key_pair.private_key);
    assert_eq!(original_key_pair.public_key, deserialized_key_pair.public_key);
}

#[test]
fn test_public_key_hash() {
    let key_pair = KeyPair::generate();
    let hash = key_pair.public_key_hash();

    // Test hash properties
    assert_eq!(hash.len(), 64);  // SHA-256 hash is 32 bytes = 64 hex chars
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit())); // All characters should be hex
}

#[test]
fn test_key_pair_uniqueness() {
    let key_pair1 = KeyPair::generate();
    let key_pair2 = KeyPair::generate();

    // Test that generated keys are unique
    assert_ne!(key_pair1.private_key, key_pair2.private_key);
    assert_ne!(key_pair1.public_key, key_pair2.public_key);
    assert_ne!(key_pair1.public_key_hash(), key_pair2.public_key_hash());
}

#[test]
fn test_key_relationship() {
    let key_pair = KeyPair::generate();
    
    // Verify that the public key is correctly derived from the private key
	let private_key_bytes = GenericArray::from_slice(&key_pair.private_key);
    let signing_key = SigningKey::from_bytes(private_key_bytes).unwrap();
    let derived_verifying_key = VerifyingKey::from(&signing_key);
    let derived_public_key = derived_verifying_key.to_sec1_bytes().to_vec();
    
    assert_eq!(key_pair.public_key, derived_public_key);
}

#[test]
fn test_json_format() {
    let key_pair = KeyPair::generate();
    let json = key_pair.to_json();
    
    // Test that JSON is valid and contains expected fields
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_object());
    assert!(parsed.as_object().unwrap().contains_key("private_key"));
    assert!(parsed.as_object().unwrap().contains_key("public_key"));
}

#[test]
#[should_panic]
fn test_invalid_json_deserialization() {
    KeyPair::from_json("invalid json");
}

#[test]
fn test_hash_consistency() {
    let key_pair = KeyPair::generate();
    let hash1 = key_pair.public_key_hash();
    let hash2 = key_pair.public_key_hash();

    // Test that hashing is deterministic
    assert_eq!(hash1, hash2);
}

#[test]
fn test_manual_keypair_creation() {
    // Create a known valid key pair for testing
    let mut rng = OsRng;
    let signing_key = SigningKey::random(&mut rng);
    let verifying_key = VerifyingKey::from(&signing_key);

    let key_pair = KeyPair {
        private_key: signing_key.to_bytes().to_vec(),
        public_key: verifying_key.to_sec1_bytes().to_vec(),
    };

    // Verify the key pair is valid
    assert_eq!(key_pair.private_key.len(), 32);
    assert_eq!(key_pair.public_key.len(), 65);
}

#[test]
fn test_large_batch_generation() {
    let mut hashes = std::collections::HashSet::new();
    for _ in 0..100 {
        let key_pair = KeyPair::generate();
        let hash = key_pair.public_key_hash();
        assert!(hashes.insert(hash), "Generated duplicate key pair");
    }
}

#[test]
fn test_json_roundtrip_preservation() {
    let original_key_pair = KeyPair::generate();
    let json1 = original_key_pair.to_json();
    let deserialized = KeyPair::from_json(&json1);
    let json2 = deserialized.to_json();
    
    // Test that multiple serialization/deserialization cycles preserve the data
    assert_eq!(json1, json2);
}
