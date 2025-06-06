use chrono::{Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::error::AppError;

pub const TOKEN_LENGTH: usize = 64;

// トークン生成関数
pub fn generate_verification_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(TOKEN_LENGTH)
        .map(char::from)
        .collect()
}

// トークンの有効期限を計算
pub fn calculate_token_expiry() -> chrono::DateTime<Utc> {
    let expires_in =
        std::env::var("EMAIL_VERIFICATION_TOKEN_EXPIRES_IN").unwrap_or_else(|_| "24h".to_string());

    let hours = if expires_in.ends_with('h') {
        expires_in[..expires_in.len() - 1]
            .parse::<i64>()
            .unwrap_or(24)
    } else {
        24
    };

    Utc::now() + Duration::hours(hours)
}

// ユーザーの検証トークンを作成・保存
pub async fn create_verification_token(
    user_id: Uuid,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<String, AppError> {
    let token = generate_verification_token();
    let expires_at = calculate_token_expiry();

    sqlx::query!(
        r#"
        UPDATE users
        SET verification_token = $1, verification_token_expires_at = $2
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

// メール検証トークンを検証
pub async fn verify_email(token: &str, pool: &PgPool) -> Result<Uuid, AppError> {
    let now = Utc::now();

    let result = sqlx::query!(
        r#"
        SELECT id
        FROM users
        WHERE verification_token = $1
          AND verification_token_expires_at > $2
          AND email_verified_at IS NULL
        "#,
        token,
        now
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    match result {
        Some(user) => {
            // トークンを消費して検証完了をマーク
            sqlx::query!(
                r#"
                UPDATE users
                SET email_verified_at = $1,
                    email_verified = true,
                    verification_token = NULL,
                    verification_token_expires_at = NULL
                WHERE id = $2
                "#,
                now,
                user.id
            )
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(e))?;

            Ok(user.id)
        }
        None => Err(AppError::BadRequest(
            "無効または期限切れの検証トークンです".to_string(),
        )),
    }
}

// ユーザーの検証状態を確認
pub async fn is_email_verified(user_id: Uuid, pool: &PgPool) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"
        SELECT email_verified_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    match result {
        Some(user) => Ok(user.email_verified_at.is_some()),
        None => Err(AppError::BadRequest("ユーザーが見つかりません".to_string())),
    }
}
