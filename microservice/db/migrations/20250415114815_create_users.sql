-- DROP TABLE IF EXISTS users;

-- CREATE TABLE users (
    --id TEXT PRIMARY KEY NOT NULL,
    --username TEXT NOT NULL UNIQUE,
    --public_key TEXT NOT NULL UNIQUE,
    --public_key_hash TEXT NOT NULL UNIQUE,
    --created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    --updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    --last_login TIMESTAMP
--);

CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 16),
    username TEXT NOT NULL UNIQUE CHECK (length(username) >= 3 AND length(username) <= 50),
    public_key TEXT NOT NULL CHECK (
        public_key GLOB '[A-Za-z0-9_-]*' AND
        public_key NOT LIKE '%==%'
    ),
    public_key_hash TEXT NOT NULL UNIQUE CHECK (
        length(public_key_hash) = 43 AND
        public_key_hash GLOB '[A-Za-z0-9_-]*' AND
        public_key_hash NOT LIKE '%==%'
    ),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP,
    CONSTRAINT updated_after_creation CHECK (updated_at >= created_at),
    CONSTRAINT valid_login_time CHECK (last_login IS NULL OR last_login >= created_at)
);

CREATE INDEX idx_users_username ON users(username);

CREATE INDEX idx_users_pubkey_hash ON users(public_key_hash);

CREATE TRIGGER update_users_timestamp
AFTER UPDATE ON users
FOR EACH ROW
BEGIN
    UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
