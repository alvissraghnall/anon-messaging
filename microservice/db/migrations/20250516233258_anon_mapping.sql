-- Add migration script here
CREATE TABLE IF NOT EXISTS anon_mappings (
    anon_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)6
);
