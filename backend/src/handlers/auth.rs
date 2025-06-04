use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    auth::{
        jwt::create_jwt_token,
        password::{hash_password, verify_password},
    },
    config::Config,
    error::AppError,
    models::{
        auth::{
            AuthResponse, LoginRequest, LogoutResponse, RefreshTokenRequest, RegisterRequest,
            UserInfo,
        },
        {RefreshToken, User, UserCredentials},
    },
    utils::{generate_secure_token, token_hash::hash_refresh_token},
};

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 409, description = "User already exists", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    // Validate input
    payload.validate()?;

    // Check if user already exists
    let existing_user =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 OR username = $2")
            .bind(&payload.email)
            .bind(&payload.username)
            .fetch_optional(&pool)
            .await?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("User already exists".to_string()));
    }

    // Hash password
    let (password_hash, salt) = hash_password(&payload.password)?;

    // Start transaction
    let mut tx = pool.begin().await?;

    // Create user
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, display_name, email_verified)
        VALUES ($1, $2, $3, false)
        RETURNING *
        "#,
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&payload.display_name)
    .fetch_one(&mut *tx)
    .await?;

    // Create user credentials
    sqlx::query(
        r#"
        INSERT INTO user_credentials (user_id, password_hash, salt)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user.id)
    .bind(&password_hash)
    .bind(&salt)
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    // Generate tokens
    let config = Config::from_env()?;
    let access_token = create_jwt_token(
        &user.id.to_string(),
        &user.username,
        &user.email,
        &config.jwt_secret,
        15, // 15 minutes
    )?;

    let refresh_token = generate_secure_token();
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

    Ok((StatusCode::CREATED, Json(response)))
}

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

    let refresh_token = generate_secure_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    println!("Generated refresh token: {}", refresh_token);
    println!("Generated refresh token hash: {}", refresh_token_hash);

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

    println!("Received refresh token: {}", payload.refresh_token);
    println!("Computed token hash: {}", token_hash);

    // Find and validate refresh token
    let refresh_token = sqlx::query_as::<_, RefreshToken>(
        r#"
        SELECT * FROM refresh_tokens 
        WHERE token_hash = $1 AND revoked = false
        "#,
    )
    // WHERE token_hash = $1 AND revoked = false AND expires_at > NOW()
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

#[utoipa::path(
    get,
    path = "/api/auth/google",
    responses(
        (status = 302, description = "Redirect to Google OAuth")
    ),
    tag = "auth"
)]
pub async fn google_auth() -> Result<Json<serde_json::Value>, AppError> {
    // TODO: Implement Google OAuth initiation with proper redirect URL
    // For now, return a placeholder response
    Ok(Json(serde_json::json!({
        "message": "Google OAuth not fully implemented yet",
        "redirect_url": "https://accounts.google.com/oauth/authorize"
    })))
}

#[utoipa::path(
    get,
    path = "/api/auth/google/callback",
    responses(
        (status = 200, description = "Google OAuth callback processed", body = AuthResponse),
        (status = 400, description = "OAuth error", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn google_callback(
    State(_pool): State<PgPool>,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AuthResponse>, AppError> {
    // TODO: Implement Google OAuth callback processing
    // This would involve exchanging the authorization code for tokens,
    // fetching user info from Google, and creating/updating the user account
    Err(AppError::NotImplemented(
        "Google OAuth callback not implemented yet".to_string(),
    ))
}
