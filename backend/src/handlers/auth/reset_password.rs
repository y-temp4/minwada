use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    auth::password,
    error::AppError,
    models::{
        auth::{MessageResponse, ResetPasswordRequest},
        common::ErrorResponse,
    },
    utils::password_reset,
};

/// Reset password
///
/// Reset user's password using a valid reset token
#[utoipa::path(
    post,
    path = "/api/auth/password-reset/{token}",
    request_body = ResetPasswordRequest,
    params(
        ("token" = String, Path, description = "Password reset token")
    ),
    responses(
        (status = 200, description = "Password reset successfully", body = MessageResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Invalid or expired token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "Auth"
)]
pub async fn reset_password(
    State(pool): State<PgPool>,
    axum::extract::Path(token): axum::extract::Path<String>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    request.validate()?;

    // トークンを検証して該当ユーザーを取得
    let user_id = password_reset::verify_reset_token(&token, &pool).await?;

    // パスワードハッシュの生成
    let (password_hash, salt) = password::hash_password(&request.new_password)?;

    // トランザクション開始
    let mut tx = pool.begin().await.map_err(|e| AppError::Database(e))?;

    // ユーザー認証情報の更新
    sqlx::query!(
        r#"
        UPDATE user_credentials
        SET password_hash = $1, salt = $2, updated_at = NOW()
        WHERE user_id = $3
        "#,
        password_hash,
        salt,
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e))?;

    // リセットトークンの消去
    sqlx::query!(
        r#"
        UPDATE users
        SET password_reset_token = NULL, password_reset_token_expires_at = NULL
        WHERE id = $1
        "#,
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e))?;

    // トランザクションのコミット
    tx.commit().await.map_err(|e| AppError::Database(e))?;

    // 成功レスポンス
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "パスワードが正常にリセットされました。新しいパスワードでログインしてください。"
        })),
    ))
}
