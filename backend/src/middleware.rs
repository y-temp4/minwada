use axum::body::Body;
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use sqlx::PgPool;

use crate::{auth::jwt::verify_jwt_token, config::Config, error::AppError, models::User};

pub async fn auth_middleware(
    State(pool): State<PgPool>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Authorizationヘッダー取得
    let headers = request.headers();

    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| {
            AppError::Unauthorized("Missing or invalid Authorization header".to_string())
        })?;

    // トークン検証
    let config = Config::from_env()?;
    let claims = verify_jwt_token(auth_header, &config.jwt_secret)?;

    // ユーザー取得
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&claims.sub)?)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

    request.extensions_mut().insert(user);
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
