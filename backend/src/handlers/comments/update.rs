use axum::{extract::Extension, extract::Path, extract::State, Json};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        comments::{CommentResponse, CommentWithUser, UpdateCommentRequest},
        common::ErrorResponse,
        User,
    },
};

#[utoipa::path(
    put,
    path = "/api/comments/{id}",
    params(
        ("id" = Uuid, Path, description = "Comment ID")
    ),
    request_body = UpdateCommentRequest,
    responses(
        (status = 200, description = "Comment updated successfully", body = CommentResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Comment not found", body = ErrorResponse)
    ),
    tag = "comments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_comment(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<Json<CommentResponse>, AppError> {
    // Validate input
    payload.validate()?;

    // Check if comment exists and user owns it
    let existing_comment = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM comments WHERE id = $1 AND user_id = $2)",
    )
    .bind(id)
    .bind(current_user.id)
    .fetch_one(&pool)
    .await?;

    if !existing_comment {
        return Err(AppError::NotFound);
    }

    // Update comment
    let updated_comment = sqlx::query_as::<_, CommentWithUser>(
        r#"
        UPDATE comments 
        SET content = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING 
            id, thread_id, content, parent_id, created_at, updated_at,
            user_id, $3 as username, $4 as user_display_name, $5 as user_avatar_url
        "#,
    )
    .bind(id)
    .bind(&payload.content)
    .bind(&current_user.username)
    .bind(&current_user.display_name)
    .bind(&current_user.avatar_url)
    .fetch_one(&pool)
    .await?;

    Ok(Json(updated_comment.to_response()))
}