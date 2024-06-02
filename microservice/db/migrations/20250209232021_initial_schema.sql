-- migrations/20231010123456_initial_schema.sql
CREATE TABLE users (
	 id INTEGER PRIMARY KEY AUTOINCREMENT,
	 user_id TEXT NOT NULL UNIQUE,  -- Unique identifier for the user
	 public_key_hash TEXT NOT NULL  -- SHA-256 hash of the public key as a hex string
);
