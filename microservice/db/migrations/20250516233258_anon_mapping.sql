
CREATE TABLE IF NOT EXISTS anon_mappings (
    anon_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    thread_id INTEGER,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (thread_id) REFERENCES messages(id)
);

CREATE INDEX IF NOT EXISTS idx_anon_mappings_user ON anon_mappings(user_id, expires_at);
CREATE INDEX IF NOT EXISTS idx_anon_mappings_thread ON anon_mappings(thread_id);
