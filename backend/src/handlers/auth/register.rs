use axum::{extract::State, http::StatusCode, Json};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    auth::{jwt::create_jwt_token, password::hash_password},
    config::Config,
    error::AppError,
    models::{
        auth::{AuthResponse, RegisterRequest, UserInfo},
        User,
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
