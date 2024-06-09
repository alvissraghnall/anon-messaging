
DROP TABLE IF EXISTS users;
CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id TEXT UNIQUE NOT NULL,
        public_key_hash TEXT NOT NULL,
        encrypted_private_key TEXT NOT NULL,
        encryption_salt TEXT NOT NULL,
        encryption_nonce TEXT NOT NULL
);




