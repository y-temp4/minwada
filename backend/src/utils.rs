// Utility functions

use uuid::Uuid;

// Generate a new UUID
pub fn generate_id() -> Uuid {
    Uuid::new_v4()
}

// Parse time durations (e.g., "15m", "7d")
pub fn parse_duration_to_minutes(duration: &str) -> Result<i64, String> {
    if duration.ends_with('m') {
        duration[..duration.len() - 1]
            .parse::<i64>()
            .map_err(|_| "Invalid minute format".to_string())
    } else if duration.ends_with('h') {
        duration[..duration.len() - 1]
            .parse::<i64>()
            .map(|h| h * 60)
            .map_err(|_| "Invalid hour format".to_string())
    } else if duration.ends_with('d') {
        duration[..duration.len() - 1]
            .parse::<i64>()
            .map(|d| d * 24 * 60)
            .map_err(|_| "Invalid day format".to_string())
    } else {
        Err("Invalid duration format. Use 'm' for minutes, 'h' for hours, 'd' for days".to_string())
    }
}

// Validate email format (basic validation)
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

// Generate a secure random string for tokens
pub fn generate_secure_token() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string().replace('-', "")
}

// リフレッシュトークンのハッシュ化と検証用のユーティリティ
pub mod token_hash {
    use base64::{engine::general_purpose, Engine as _};
    use sha2::{Digest, Sha256};

    /// リフレッシュトークンをハッシュ化する関数
    /// SHA-256を使用して一貫したハッシュを生成します（ソルトなし）
    pub fn hash_refresh_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token);
        let result = hasher.finalize();
        general_purpose::STANDARD.encode(result)
    }

    /// リフレッシュトークンを比較する関数
    pub fn verify_refresh_token(token: &str, hash: &str) -> bool {
        let token_hash = hash_refresh_token(token);
        token_hash == hash
    }
}
