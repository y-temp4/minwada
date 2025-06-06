use axum::{extract::State, Json};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    auth::{jwt::create_jwt_token, password::verify_password},
    config::Config,
    error::AppError,
    models::{
        auth::{AuthResponse, LoginRequest, UserInfo},
        common::ErrorResponse,
        User, UserCredentials,
    },
    utils::{self, token_hash::hash_refresh_token},
};

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate input
    payload.validate()?;

    // Find user by email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    // Get user credentials
    let credentials =
        sqlx::query_as::<_, UserCredentials>("SELECT * FROM user_credentials WHERE user_id = $1")
            .bind(user.id)
            .fetch_optional(&pool)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    // Verify password
    if !verify_password(&payload.password, &credentials.password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate tokens
    let config = Config::from_env()?;
    let access_token = create_jwt_token(
        &user.id.to_string(),
        &user.username,
        &user.email,
        &config.jwt_secret,
        15, // 15 minutes
    )?;

    let refresh_token = utils::generate_secure_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    // Store refresh token
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, NOW() + INTERVAL '7 days')
        "#,
    )
    .bind(user.id)
    .bind(&refresh_token_hash)
    .execute(&pool)
    .await?;

    let response = AuthResponse {
        access_token,
        refresh_token,
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
