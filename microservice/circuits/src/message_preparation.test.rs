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

