use ark_crypto_primitives::crh::{poseidon::CRH as PoseidonCRH, CRHScheme};
use ark_ff::PrimeField;
use ark_relations::lc;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_std::vec::Vec;

#[derive(Clone)]
pub struct MessageReceptionCircuit<F: PrimeField> {
    pub message: Option<F>,             // Private input (message)
    pub recipient_sk: Option<F>,        // Private input (secret key)
    pub expected_commitment: Option<F>, // Public input (expected commitment)
}

impl<F: PrimeField> ConstraintSynthesizer<F> for MessageReceptionCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // Allocate variables in the circuit
        let message_var =
            cs.new_witness_variable(|| self.message.ok_or(SynthesisError::AssignmentMissing))?;
        let recipient_sk_var =
            cs.new_witness_variable(|| self.recipient_sk.ok_or(SynthesisError::AssignmentMissing))?;
        let expected_commitment_var = cs.new_input_variable(|| {
            self.expected_commitment
                .ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Compute the commitment using Poseidon hash
        let computed_commitment = PoseidonCRH::evaluate(&[message_var, recipient_sk_var])?;

        // Constraint: computed_commitment must match expected_commitment
        cs.enforce_constraint(
            lc!() + computed_commitment,
            lc!() + CS::one(),
            lc!() + expected_commitment_var,
        )?;

        Ok(())
    }
}
