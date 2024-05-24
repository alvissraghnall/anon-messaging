use ark_ec::{CurveGroup, PrimeGroup};
use ark_ed25519::{EdwardsProjective as Ed25519, Fr};
use ark_ff::UniformRand;
use ark_std::rand::thread_rng;
use std::ops::Mul;

#[cfg(test)]
#[path = "message_preparation.test.rs"]
mod tests;

struct UserB {
    esk: Fr,
    epub: Ed25519,
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
}
