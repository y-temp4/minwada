use axum::{
    extract::State,
    Json,
};
use sqlx::PgPool;

use crate::{
    auth::jwt::create_jwt_token,
    config::Config,
    error::AppError,
    models::{
        auth::{AuthResponse, RefreshTokenRequest, UserInfo},
        common::ErrorResponse,
        RefreshToken, User,
    },
    utils::{generate_secure_token, token_hash::hash_refresh_token},
};

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = AuthResponse),
        (status = 401, description = "Invalid refresh token", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn refresh_token(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let token_hash = hash_refresh_token(&payload.refresh_token);

    // Find and validate refresh token
    let refresh_token = sqlx::query_as::<_, RefreshToken>(
        r#"
        SELECT * FROM refresh_tokens 
        WHERE token_hash = $1 AND revoked = false AND expires_at > NOW()
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    // Get user information
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(refresh_token.user_id)
        .fetch_one(&pool)
        .await?;

    // Generate new access token
    let config = Config::from_env()?;
    let access_token = create_jwt_token(
        &user.id.to_string(),
        &user.username,
        &user.email,
        &config.jwt_secret,
        15, // 15 minutes
    )?;

    // Generate new refresh token
    let new_refresh_token = generate_secure_token();
    let new_refresh_token_hash = hash_refresh_token(&new_refresh_token);

    // Revoke old refresh token and create new one
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE id = $1")
        .bind(refresh_token.id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, NOW() + INTERVAL '7 days')
        "#,
    )
    .bind(user.id)
    .bind(&new_refresh_token_hash)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let response = AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 900, // 15 minutes in seconds
        user: UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            email_verified: user.email_verified,
            created_at: user.created_at,
        },
    };

    Ok(Json(response))
}
