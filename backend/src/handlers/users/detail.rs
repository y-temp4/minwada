use axum::{
    extract::{Path, State},
    response::Json,
};
use sqlx::PgPool;

use crate::{
    error::AppError,
    models::{common::ErrorResponse, users::PublicUserResponse, User},
};

#[utoipa::path(
    get,
    path = "/api/users/{username}",
    params(
        ("username" = String, Path, description = "Username to lookup")
    ),
    responses(
        (status = 200, description = "User profile found", body = PublicUserResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    tag = "users"
)]
pub async fn get_user_by_username(
    State(pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<PublicUserResponse>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(PublicUserResponse::from(user)))
}
