use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        auth::{MessageResponse, RequestPasswordResetRequest},
        common::ErrorResponse,
    },
    utils::{email_sender::send_password_reset_email, password_reset},
};

/// Request a password reset
///
/// Request to reset a user's password by sending an email with a reset link.
#[utoipa::path(
    post,
    path = "/api/auth/password-reset/request",
    request_body = RequestPasswordResetRequest,
    responses(
        (status = 200, description = "Password reset email sent successfully", body = MessageResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "Auth"
)]
pub async fn request_password_reset(
    State(pool): State<PgPool>,
    Json(request): Json<RequestPasswordResetRequest>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    request.validate()?;

    // メールアドレスからユーザーを検索
    let user = sqlx::query!(
        r#"
        SELECT 
            id, username, email, display_name, avatar_url, 
            email_verified::bool as "email_verified!",
            email_verified_at,
            verification_token,
            verification_token_expires_at,
            created_at as "created_at!", updated_at as "updated_at!"
        FROM users
        WHERE email = $1
        "#,
        request.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    match user {
        Some(user) => {
            // ユーザーが存在する場合はパスワードリセットトークンを生成し、メールを送信
            let mut tx = pool.begin().await.map_err(|e| AppError::Database(e))?;

            let user_model = crate::models::User {
                id: user.id,
                username: user.username,
                email: user.email,
                display_name: user.display_name,
                avatar_url: user.avatar_url,
                email_verified: user.email_verified,
                email_verified_at: user.email_verified_at,
                verification_token: user.verification_token,
                verification_token_expires_at: user.verification_token_expires_at,
                password_reset_token: None,
                password_reset_token_expires_at: None,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };

            // パスワードリセットトークンの生成
            let reset_token = password_reset::create_reset_token(user_model.id, &mut tx).await?;

            // トランザクションのコミット
            tx.commit().await.map_err(|e| AppError::Database(e))?;

            // メール送信
            send_password_reset_email(&user_model, &reset_token).await?;

            // 成功レスポンス（セキュリティのため、ユーザーが見つからない場合と同じメッセージを返す）
            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "パスワードリセット用のメールを送信しました。メールに記載されたリンクからパスワードを再設定してください。"
                })),
            ))
        }
        None => {
            // セキュリティ上の理由から、ユーザーが存在しない場合でも成功を装う
            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "パスワードリセット用のメールを送信しました。メールに記載されたリンクからパスワードを再設定してください。"
                })),
            ))
        }
    }
}
