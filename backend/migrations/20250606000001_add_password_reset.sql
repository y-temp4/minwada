-- パスワードリセット用のカラムを追加
ALTER TABLE users ADD COLUMN password_reset_token VARCHAR(128);
ALTER TABLE users ADD COLUMN password_reset_token_expires_at TIMESTAMP WITH TIME ZONE;

-- インデックスを追加
CREATE INDEX idx_users_password_reset_token ON users(password_reset_token);
