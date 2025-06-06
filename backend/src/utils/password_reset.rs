use chrono::{Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::error::AppError;

pub const TOKEN_LENGTH: usize = 64;

// トークン生成関数
pub fn generate_reset_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(TOKEN_LENGTH)
        .map(char::from)
        .collect()
}

// トークンの有効期限を計算
pub fn calculate_token_expiry() -> chrono::DateTime<Utc> {
    let expires_in =
        std::env::var("PASSWORD_RESET_TOKEN_EXPIRES_IN").unwrap_or_else(|_| "1h".to_string());

    let hours = if expires_in.ends_with('h') {
        expires_in[..expires_in.len() - 1]
            .parse::<i64>()
            .unwrap_or(1)
    } else {
        1
    };

    Utc::now() + Duration::hours(hours)
}

// ユーザーのリセットトークンを作成・保存
pub async fn create_reset_token(
    user_id: Uuid,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<String, AppError> {
    let token = generate_reset_token();
    let expires_at = calculate_token_expiry();

    sqlx::query!(
        r#"
        UPDATE users
        SET password_reset_token = $1, password_reset_token_expires_at = $2
        WHERE id = $3
        "#,
        token,
        expires_at,
        user_id
    )
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::Database(e))?;

    Ok(token)
}

// パスワードリセットトークンを検証
pub async fn verify_reset_token(token: &str, pool: &PgPool) -> Result<Uuid, AppError> {
    let now = Utc::now();

    let result = sqlx::query!(
        r#"
        SELECT id
        FROM users
        WHERE password_reset_token = $1
          AND password_reset_token_expires_at > $2
        "#,
        token,
        now
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    match result {
        Some(user) => Ok(user.id),
        None => Err(AppError::BadRequest(
            "無効または期限切れのリセットトークンです".to_string(),
        )),
    }
}
