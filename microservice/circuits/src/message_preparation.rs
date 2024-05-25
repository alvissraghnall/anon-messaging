use ark_ec::{CurveGroup, PrimeGroup};
use ark_ed25519::{EdwardsProjective as Ed25519, Fr};
use ark_ff::UniformRand;
use ark_serialize::CanonicalSerialize;
use ark_std::rand::thread_rng;
use sha2::{Digest, Sha256};
use std::ops::Mul;

#[cfg(test)]
#[path = "message_preparation.test.rs"]
mod tests;

struct UserB {
    esk: Fr,
    epub: Ed25519,
}

#[derive(Debug)]
struct DerivedKeys {
    randomness: [u8; 32], // r for commitment
    enc_key: [u8; 32],    // K for encryption
}

impl UserB {
    fn generate_ephemeral_keypair() -> Self {
        // Initialize random number generator
        let mut rng = thread_rng();

        // Generate ephemeral secret key (esk)
        let esk = Fr::rand(&mut rng);

        // Compute ephemeral public key (epub)
        // Get the generator point G and multiply by esk
        let generator = Ed25519::generator();
        let epub = generator.mul(esk);

        Self { esk, epub }
    }

    // Compute shared secret using User A's public key
    fn compute_shared_secret(&self, pk_a: &Ed25519) -> Ed25519 {
        // Multiply User A's public key by our ephemeral secret key
        pk_a.mul(self.esk)
    }

    fn derive_keys(&self, shared_secret: &Ed25519) -> DerivedKeys {
        // Serialize the shared secret point to bytes
        let mut ss_bytes = Vec::new();
        shared_secret
            .serialize_uncompressed(&mut ss_bytes)
            .expect("Serialization failed");

        // Create two separate hasher instances for domain separation
        let mut hasher_r = Sha256::new();
        let mut hasher_k = Sha256::new();

        // Add domain separation tags before the shared secret
        hasher_r.update(b"randomness");
        hasher_r.update(&ss_bytes);

        hasher_k.update(b"encryption_key");
        hasher_k.update(&ss_bytes);

        // Finalize both hashes
        let randomness = hasher_r.finalize();
        let enc_key = hasher_k.finalize();

        // Convert the generic arrays to fixed-size arrays
        let mut r_bytes = [0u8; 32];
        let mut k_bytes = [0u8; 32];

        r_bytes.copy_from_slice(&randomness[..32]);
        k_bytes.copy_from_slice(&enc_key[..32]);

        DerivedKeys {
            randomness: r_bytes,
            enc_key: k_bytes,
        }
    }
}
