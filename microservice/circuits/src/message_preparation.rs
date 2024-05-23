use ark_ec::{CurveGroup, PrimeGroup};
use ark_ed25519::{EdwardsProjective as Ed25519, Fr};
use ark_ff::UniformRand;
use ark_std::rand::thread_rng;
use std::ops::Mul;

#[cfg(test)]
#[path = "message_preparation.test.rs"]
mod tests;

fn generate_ephemeral_keypair() -> (Fr, Ed25519) {
    // Initialize random number generator
    let mut rng = thread_rng();

    // Generate ephemeral secret key (esk)
    let esk = Fr::rand(&mut rng);

    // Compute ephemeral public key (epub)
    // Get the generator point G and multiply by esk
    let generator = Ed25519::generator();
    let epub = generator.mul(esk);

    (esk, epub)
}
