-- Add migration script here
ALTER TABLE users
ADD COLUMN email_verified_at TIMESTAMPTZ,
ADD COLUMN verification_token VARCHAR(255),
ADD COLUMN verification_token_expires_at TIMESTAMPTZ;

CREATE INDEX idx_users_verification_token ON users(verification_token);
