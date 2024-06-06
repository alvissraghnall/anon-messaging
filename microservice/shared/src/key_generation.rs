use db::db::{create_db_pool, insert_user};
use p256::ecdsa::{SigningKey, VerifyingKey};
//use rand::rngs::OsRng;
use rand_core::OsRng;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize)]
pub struct KeyPair {
	private_key: Vec<u8>,
	public_key: Vec<u8>, 
}

impl KeyPair {
	 /// Generates a new ECC key pair.
	 pub fn generate() -> Self {
	     let mut rng = OsRng;
	     let signing_key = SigningKey::random(&mut rng);  // Private key
	     let verifying_key = VerifyingKey::from(&signing_key);  // Public key

	     KeyPair {
	         private_key: signing_key.to_bytes().to_vec(),
	         public_key: verifying_key.to_sec1_bytes().to_vec(),
	     }
	 }

	 /// Returns the private key as a byte vector.
	 pub fn private_key(&self) -> &[u8] {
	     &self.private_key
	 }

	 /// Returns the public key as a byte vector.
	 pub fn public_key(&self) -> &[u8] {
	     &self.public_key
	 }
     /// Serializes the key pair to a JSON string.
     pub fn to_json(&self) -> String {
         serde_json::to_string(self).unwrap()
     }

     /// Deserializes the key pair from a JSON string.
     pub fn from_json(json: &str) -> Self {
         serde_json::from_str(json).unwrap()
	 }
	
     /// Returns the SHA-256 hash of the public key as a hex string.
     pub fn public_key_hash(&self) -> String {
         let mut hasher = Sha256::new();
         hasher.update(&self.public_key);
         let result = hasher.finalize();
         format!("{:x}", result)
     }

}


#[cfg(test)]
#[path = "key_generation.test.rs"]
mod tests;
