use axum::{
    extract::State,
    Json,
};
use sqlx::PgPool;

use crate::{
    error::AppError,
    models::auth::{LogoutResponse, RefreshTokenRequest},
    utils::token_hash::hash_refresh_token,
};

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "Logout successful", body = LogoutResponse),
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn logout(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<LogoutResponse>, AppError> {
    // Hash the provided refresh token to find it in database
    let token_hash = hash_refresh_token(&payload.refresh_token);

    // Revoke refresh token
    sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE token_hash = $1")
        .bind(&token_hash)
        .execute(&pool)
        .await?;

    Ok(Json(LogoutResponse {
        message: "Logged out successfully".to_string(),
    }))
}
