use super::*;
use std::collections::HashSet;
use ark_std::Zero;

#[test]
fn test_generate_ephemeral_keypair() {
    let (esk, epub) = generate_ephemeral_keypair();

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
        let (esk, epub) = generate_ephemeral_keypair();
        assert!(esk_set.insert(esk));
        assert!(epub_set.insert(epub));
    }
}

#[test]
fn test_public_key_on_curve() {
    let (_, epub) = generate_ephemeral_keypair();

    // Check that epub is a valid Ed25519 point
    let affine_epub = epub.into_affine();

    assert!(affine_epub.is_on_curve());
    assert!(affine_epub.is_in_correct_subgroup_assuming_on_curve());
}
