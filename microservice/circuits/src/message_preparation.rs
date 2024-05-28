use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use ark_ec::{AffineRepr, CurveGroup, PrimeGroup};
use ark_ed25519::{EdwardsAffine, EdwardsProjective as Ed25519, Fr};
use ark_ff::{UniformRand, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::thread_rng;
use ark_std::Zero;
use blake2::Blake2b512;
use sha2::{Digest, Sha256};
use std::ops::Mul;
use subtle::{Choice, ConstantTimeEq};

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

#[derive(Debug)]
struct Ciphertext {
    nonce: [u8; 12],    // 96-bit nonce for AES-GCM
    encrypted: Vec<u8>, // Encrypted message with authentication tag
}

#[derive(Clone, Debug)]
pub struct PedersenParams {
    /// Base generator G
    pub g: EdwardsAffine,
    /// Secondary generator H
    pub h: EdwardsAffine,
}

/// Commitment output containing the commitment point and auxiliary information
#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct PedersenCommitment {
    /// The commitment point on the curve
    pub commitment: Ed25519,
    /// Randomness used (stored for opening)
    pub randomness: Fr,
}

#[derive(Debug, thiserror::Error)]
pub enum CommitmentError {
    #[error("Invalid randomness value")]
    InvalidRandomness,

    #[error("Error in hashing operation")]
    HashingError,

    #[error("Invalid commitment parameters")]
    InvalidParameters,
}

impl PedersenParams {
    /// Initialize Pedersen parameters using standardized generators
    pub fn new() -> Self {
        // Base generator G is the standard Ed25519 generator
        let g = EdwardsAffine::generator();

        // H is derived using the SHA512 hash of "ed25519_pedersen_h" || G
        // This is a nothing-up-my-sleeve point generation
        let h = Self::generate_h(&g);
        // let h = Self::generate_alternate_generator(&g);

        Self { g, h }
    }

    /// Check if point has large order (not in small subgroup)
    fn has_large_order(point: &EdwardsAffine) -> bool {
        // Ed25519 cofactor is 8
        let eight = Fr::from(8u64);
        let eight_times_point = Ed25519::from(*point).mul(eight);
        !eight_times_point.is_zero()
    }

    /// Hash to curve implementation following RFC 9380
    fn hash_to_curve(hash: &[u8]) -> Option<EdwardsAffine> {
        // In production, use a proper hash-to-curve implementation
        // This is a placeholder showing the concept
        let mut field_elem = Fr::zero();
        if Fr::deserialize_compressed(hash).is_ok() {
            let point = Ed25519::generator().mul(field_elem);
            Some(point.into_affine())
        } else {
            None
        }
    }

    /// Generate the secondary generator H using nothing-up-my-sleeve process
    fn generate_h(g: &EdwardsAffine) -> EdwardsAffine {
        let mut g_bytes = Vec::new();
        g.serialize_uncompressed(&mut g_bytes)
            .expect("Serialization of base point failed");

        let mut counter = 0u64;
        loop {
            let mut hasher = Blake2b512::new();
            hasher.update(b"ed25519_pedersen_h");
            hasher.update(&g_bytes);
            hasher.update(&counter.to_le_bytes());
            let hash = hasher.finalize();

            // Try to map hash to curve point
            if let Some(h) = Self::hash_to_curve(&hash) {
                // Verify h is not the identity and not a small order point
                if !h.is_zero() && Self::has_large_order(&h) {
                    return h;
                }
            }
            counter += 1;
        }
    }

    fn generate_alternate_generator(g: &EdwardsAffine) -> EdwardsAffine {
        let mut hasher = Blake2b512::new();
        hasher.update(b"ed25519_pedersen_h_v1");
        
        // Add generator coordinates to the hash
        let mut g_bytes = Vec::new();
        g.serialize_uncompressed(&mut g_bytes)
            .expect("Serialization of base point failed");
        hasher.update(&g_bytes);
        
        // Use the hash output to create a scalar
        let hash = hasher.finalize();
        let mut scalar = Fr::deserialize_compressed(&hash[..32])
            .expect("Hash to scalar failed");
        
        // Multiply generator by scalar and clear cofactor
        // This ensures the point has the right order
        let h_projective = Ed25519::generator().mul(scalar);
        let h = h_projective.into_affine();
        
        // Verify point is valid (non-zero and correct order)
        assert!(!h.is_zero(), "Generated point must not be identity");
        
        h
    }
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

    fn encrypt_message(
        &self,
        message: &[u8],
        key: &[u8; 32],
    ) -> Result<Ciphertext, Box<dyn std::error::Error>> {
        // Create a new cipher instance
        let cipher = Aes256Gcm::new(key.as_slice().into());

        // Generate a random nonce
        let mut rng = thread_rng();
        let mut nonce = [0u8; 12];
        for byte in &mut nonce {
            *byte = UniformRand::rand(&mut rng);
        }
        let nonce = Nonce::from_slice(&nonce);

        // Encrypt the message
        let encrypted = cipher
            .encrypt(nonce, message)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        Ok(Ciphertext {
            nonce: nonce.to_owned().into(),
            encrypted,
        })
    }

    // SHOULD NOT BE HERE!!!!!!!!
    fn decrypt_message(
        &self,
        ciphertext: &Ciphertext,
        key: &[u8; 32],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let cipher = Aes256Gcm::new(key.as_slice().into());
        let nonce = Nonce::from_slice(&ciphertext.nonce);

        let decrypted = cipher
            .decrypt(nonce, ciphertext.encrypted.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;

        Ok(decrypted)
    }

    /// Compute the commitment in constant time
    fn compute_commitment(m: &Fr, r: &Fr, params: &PedersenParams) -> Ed25519 {
        let g_m = Ed25519::from(params.g).mul_bigint(m.into_bigint());
        let h_r = Ed25519::from(params.h).mul_bigint(r.into_bigint());
        g_m + h_r
    }

    /// Hash ciphertext to scalar using domain separation
    fn hash_ciphertext_to_scalar(ciphertext: &Ciphertext) -> Result<Fr, CommitmentError> {
        let mut hasher = Blake2b512::new();

        // Domain separation
        hasher.update(b"PEDERSEN_COMMITMENT_V1");

        // Hash structure of ciphertext
        hasher.update(&(ciphertext.nonce.len() as u64).to_le_bytes());
        hasher.update(&ciphertext.nonce);
        hasher.update(&(ciphertext.encrypted.len() as u64).to_le_bytes());
        hasher.update(&ciphertext.encrypted);

        let hash = hasher.finalize();

        let mut scalar = Fr::deserialize_compressed(&hash[..32])
            .map_err(|_| CommitmentError::HashingError)?;

        Ok(scalar)
    }

    /// Convert randomness bytes to scalar in constant time
    fn randomness_to_scalar(randomness: &[u8; 32]) -> Result<Fr, CommitmentError> {
        let mut scalar = Fr::deserialize_compressed(&randomness[..])
            .map_err(|_| CommitmentError::InvalidRandomness)?;
            
        // Ensure scalar is in proper range
        if scalar.is_zero() {
            return Err(CommitmentError::InvalidRandomness);
        }
        
        Ok(scalar)
    }

    /// Create a Pedersen commitment to a ciphertext
    /// 
    /// # Arguments
    /// * `ciphertext` - The ciphertext to commit to
    /// * `randomness` - Random value for hiding
    /// * `params` - Pedersen commitment parameters
    /// 
    /// # Returns
    /// * `Result<PedersenCommitment, CommitmentError>` - The commitment or error
    pub fn create_commitment(
        &self,
        ciphertext: &Ciphertext,
        randomness: &[u8; 32],
        params: &PedersenParams,
    ) -> Result<PedersenCommitment, CommitmentError> {
        // Convert randomness to scalar in constant time
        let r = Self::randomness_to_scalar(randomness)?;
        
        // Hash ciphertext to scalar
        let m = Self::hash_ciphertext_to_scalar(ciphertext)?;
        
        // Compute commitment: com = m * G + r * H
        let commitment = Self::compute_commitment(&m, &r, params);
        
        Ok(PedersenCommitment {
            commitment,
            randomness: r,
        })
    }

    /// Verify a commitment opening
    pub fn verify_commitment(
        &self,
        commitment: &PedersenCommitment,
        ciphertext: &Ciphertext,
        params: &PedersenParams,
    ) -> Result<bool, CommitmentError> {
        let m = Self::hash_ciphertext_to_scalar(ciphertext)?;
        let expected = Self::compute_commitment(&m, &commitment.randomness, params);
        
        Ok(expected.into_affine().eq(&commitment.commitment.into_affine()))
    }
}
