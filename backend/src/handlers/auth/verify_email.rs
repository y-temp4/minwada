use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

use crate::{
    error::AppError,
    models::{common::ErrorResponse, User},
    utils::email_verification,
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifyEmailResponse {
    message: String,
    verified: bool,
}

/// メールアドレス検証
#[utoipa::path(
    post,
    path = "/api/auth/verify-email/{token}",
    tag = "認証",
    responses(
        (status = 200, description = "メールアドレス検証が成功", body = VerifyEmailResponse),
        (status = 400, description = "無効なトークン", body = ErrorResponse),
        (status = 500, description = "サーバーエラー", body = ErrorResponse),
    ),
    params(
        ("token" = String, Path, description = "検証トークン"),
    )
)]
pub async fn verify_email(
    State(pool): State<PgPool>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = email_verification::verify_email(&token, &pool).await?;
    
    Ok((
        StatusCode::OK,
        Json(VerifyEmailResponse {
            message: "メールアドレスが正常に検証されました".to_string(),
            verified: true,
        }),
    ))
}

/// 検証メールの再送信リクエスト
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResendVerificationResponse {
    message: String,
}

/// 検証メール再送信
#[utoipa::path(
    post,
    path = "/api/auth/resend-verification",
    tag = "認証",
    responses(
        (status = 200, description = "検証メールを再送信しました", body = ResendVerificationResponse),
        (status = 400, description = "不正なリクエスト", body = ErrorResponse),
        (status = 404, description = "ユーザーが見つかりません", body = ErrorResponse),
        (status = 500, description = "サーバーエラー", body = ErrorResponse),
    ),
    security(
        ("jwt_auth" = [])
    )
)]
pub async fn resend_verification(
    State(pool): State<PgPool>,
    Extension(current_user): Extension<User>,
) -> Result<impl IntoResponse, AppError> {
    crate::utils::email_sender::resend_verification_email(current_user.id, &pool).await?;
    
    Ok((
        StatusCode::OK,
        Json(ResendVerificationResponse {
            message: "検証メールを再送信しました".to_string(),
        }),
    ))
}
