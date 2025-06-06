use lazy_static::lazy_static;
use regex::Regex;
use uuid::Uuid;

lazy_static! {
    static ref EMAIL_REGEX: Regex =
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$").unwrap();
}

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
    // 正規表現を使用してメールアドレスを検証
    EMAIL_REGEX.is_match(email)
}

// Generate a secure random string for tokens
pub fn generate_secure_token() -> String {
    Uuid::new_v4().to_string().replace('-', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2); // UUIDは一意であることを確認
    }

    #[test]
    fn test_parse_duration_to_minutes() {
        // 分単位のテスト
        assert_eq!(parse_duration_to_minutes("30m").unwrap(), 30);

        // 時間単位のテスト
        assert_eq!(parse_duration_to_minutes("2h").unwrap(), 120);

        // 日単位のテスト
        assert_eq!(parse_duration_to_minutes("1d").unwrap(), 1440);

        // 無効なフォーマットのテスト
        assert!(parse_duration_to_minutes("30").is_err());
        assert!(parse_duration_to_minutes("invalid").is_err());
    }

    #[test]
    fn test_is_valid_email() {
        // 有効なメールアドレス
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.co.jp"));
        assert!(is_valid_email("user+tag@example.com"));
        assert!(is_valid_email("user-name@example-domain.com"));
        assert!(is_valid_email("123@numbers.com"));

        // 無効なメールアドレス
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("invalid@"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@domain")); // TLDがない
        assert!(!is_valid_email("user@.com")); // ドメイン名がない
        assert!(!is_valid_email("user@domain..com")); // 連続したドット
        assert!(!is_valid_email("user name@domain.com")); // スペースを含む
    }

    #[test]
    fn test_generate_secure_token() {
        let token1 = generate_secure_token();
        let token2 = generate_secure_token();

        assert_ne!(token1, token2); // トークンは一意であることを確認
        assert!(!token1.contains('-')); // ハイフンが含まれていないことを確認
    }
}
