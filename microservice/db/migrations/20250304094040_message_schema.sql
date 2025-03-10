
CREATE TABLE encrypted_messages (
    id SERIAL PRIMARY KEY,
    sender_id TEXT NOT NULL,
    recipient_id TEXT NOT NULL,
    encrypted_message TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_sender
        FOREIGN KEY (sender_id)
        REFERENCES users(user_id)
        ON DELETE CASCADE,
    CONSTRAINT fk_recipient
        FOREIGN KEY (recipient_id)
        REFERENCES users(user_id)
        ON DELETE CASCADE
);
