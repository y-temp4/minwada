// リフレッシュトークンのハッシュ化と検証用のユーティリティ
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_refresh_token() {
        let token = "test_token";
        let hash1 = hash_refresh_token(token);
        let hash2 = hash_refresh_token(token);

        // 同じ入力に対して常に同じハッシュを返すことを確認
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_verify_refresh_token() {
        let token = "test_token";
        let hash = hash_refresh_token(token);

        // 検証が正しく機能することを確認
        assert!(verify_refresh_token(token, &hash));
        assert!(!verify_refresh_token("wrong_token", &hash));
    }
}
