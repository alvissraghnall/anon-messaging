CREATE TABLE users (
	 id SERIAL PRIMARY KEY,
	 public_key_hash TEXT NOT NULL  -- SHA-256 hash of the public key as a hex string
);
