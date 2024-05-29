use ark_ff::BigInteger;
use ark_ec::CurveGroup;
use ark_relations::lc;
//use ark_ec::{AffineCurve, ProjectiveCurve};
use ark_ff::Field;
use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_std::{rand::Rng, UniformRand};
use ark_ed25519::{EdwardsAffine, EdwardsConfig, EdwardsProjective as Ed25519, Fr, Fq};
use ark_ec::PrimeGroup;
use std::ops::Mul;

// Structure to hold the circuit parameters
pub struct MessageProtocolCircuit {
    // User A's public key
    pub pk_a: Option<Ed25519>,
    // User B's ephemeral secret key
    pub esk: Option<Fr>,
    // User B's ephemeral public key
    pub epub: Option<Ed25519>,
    // Computed shared secret
    pub shared_secret: Option<Ed25519>,
}

impl MessageProtocolCircuit {
	/// Converts a Fq element to Fr (for circuit constraints)
    fn fq_to_fr(fq: Fq) -> Fr {
        // Convert to bytes and then to Fr
        // Note: This is a simplified conversion and may need additional checks
        let bytes = fq.into_bigint().to_bytes_le();
        Fr::from_le_bytes_mod_order(&bytes)
    }

    // Function to generate new ephemeral keys
    pub fn generate_ephemeral_keys<R: Rng>(rng: &mut R) -> (Fr, Ed25519) {

		// Generate ephemeral secret key (esk)
        let esk = Fr::rand(rng);

        // Compute ephemeral public key (epub)
        // Get the generator point G and multiply by esk
		let generator = Ed25519::generator();
        let epub = generator.mul(esk).into();

        (esk, epub)
    }
    
    /// Computes the shared secret between User B's ephemeral secret key and User A's public key
    /// Returns the shared secret point on the curve
    pub fn compute_shared_secret(esk: &Fr, pk_a: &Ed25519) -> Ed25519 {
        // Perform scalar multiplication: esk * pk_a
        pk_a.mul(*esk).into()
    }

	/// Creates a new instance of the circuit with the given parameters
    pub fn new(
        pk_a: Option<Ed25519>,
        esk: Option<Fr>,
        epub: Option<Ed25519>,
        shared_secret: Option<Ed25519>,
    ) -> Self {
        Self {
            pk_a,
            esk,
            epub,
            shared_secret,
        }
    }

}

impl ConstraintSynthesizer<Fr> for MessageProtocolCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {

		// Allocate variables for the circuit
        let esk_var = cs.new_witness_variable(|| self.esk.ok_or(SynthesisError::AssignmentMissing))?;
        
        // Convert curve points to their affine representation
        let pk_a_affine = self.pk_a.map(|p| p.into_affine());
        let epub_affine = self.epub.map(|p| p.into_affine());
        let shared_secret_affine = self.shared_secret.map(|p| p.into_affine());

		// Handle coordinates as field elements in the base field (Fq)
        let pk_a_coords = cs.new_input_variable(|| {
            let point = pk_a_affine.ok_or(SynthesisError::AssignmentMissing)?;
			Ok(Self::fq_to_fr(point.x))
        })?;

		let epub_coords = cs.new_input_variable(|| {
            let point = self.epub.ok_or(SynthesisError::AssignmentMissing)?.into_affine();
            Ok(Self::fq_to_fr(point.x))
        })?;

        let shared_secret_coords = cs.new_input_variable(|| {
            let point = self.shared_secret.ok_or(SynthesisError::AssignmentMissing)?.into_affine();
            Ok(Self::fq_to_fr(point.x))
        })?;

		Ok(())
    }
}

// Test module
#[cfg(test)]
mod tests {
    use super::*;
    use ark_relations::r1cs::ConstraintSystem;
    use ark_bls12_381::Fr as Bls12_381Fr;
    use ark_std::test_rng;

    #[test]
    fn test_message_protocol_circuit() {
        let mut rng = test_rng();
        
        // Generate User A's keys (in practice, these would be provided)
        let pk_a = Bls12_381Fr::rand(&mut rng);
        
        // Generate User B's ephemeral keys
        let (esk, epub) = MessageProtocolCircuit::<Bls12_381Fr>::generate_ephemeral_keys(&mut rng);
        
        // Compute shared secret
        let shared_secret = MessageProtocolCircuit::<Bls12_381Fr>::compute_shared_secret(&esk, &pk_a);
        
        // Create circuit instance
        let circuit = MessageProtocolCircuit {
            pk_a: Some(pk_a),
            esk: Some(esk),
            epub: Some(epub),
            shared_secret: Some(shared_secret),
        };
        
        // Test constraint satisfaction
        let cs = ConstraintSystem::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(cs.is_satisfied().unwrap());
    }
}
