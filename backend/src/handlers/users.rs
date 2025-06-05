use axum::{
    extract::{Extension, Path, State},
    response::Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    models::comments::CommentListResponse,
    models::threads::ThreadListResponse,
    models::{
        common::ErrorResponse,
        users::{PublicUserResponse, UpdateProfileRequest, UserResponse},
        Comment, Thread, User,
    },
};

#[utoipa::path(
    get,
    path = "/api/users/me",
    responses(
        (status = 200, description = "Current user profile", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_user(
    Extension(current_user): Extension<User>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(UserResponse::from(current_user)))
}

#[utoipa::path(
    put,
    path = "/api/users/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Username already exists", body = ErrorResponse)
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_profile(
    State(pool): State<PgPool>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Validate input
    payload.validate()?;

    // Check if username is taken by another user
    if let Some(ref username) = payload.username {
        let existing_user = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username = $1 AND id != $2",
        )
        .bind(username)
        .bind(current_user.id)
        .fetch_one(&pool)
        .await?;

        if existing_user > 0 {
            return Err(AppError::Conflict("Username already exists".to_string()));
        }
    }

    if payload.username.is_none() && payload.display_name.is_none() && payload.avatar_url.is_none()
    {
        return Err(AppError::BadRequest("No fields to update".to_string()));
    }

    // Update user with simplified query
    let updated_user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users 
        SET 
            username = COALESCE($2, username),
            display_name = COALESCE($3, display_name),
            avatar_url = COALESCE($4, avatar_url),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(current_user.id)
    .bind(payload.username.as_ref())
    .bind(payload.display_name.as_ref())
    .bind(payload.avatar_url.as_ref())
    .fetch_one(&pool)
    .await?;

    Ok(Json(UserResponse::from(updated_user)))
}

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
