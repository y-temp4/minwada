use axum::{
    extract::{Extension, Path, State},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        common::ErrorResponse,
        threads::{ThreadResponse, ThreadWithUser, UpdateThreadRequest},
        User,
    },
};

#[utoipa::path(
    put,
    path = "/api/threads/{id}",
    params(
        ("id" = Uuid, Path, description = "Thread ID")
    ),
    request_body = UpdateThreadRequest,
    responses(
        (status = 200, description = "Thread updated successfully", body = ThreadResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Thread not found", body = ErrorResponse)
    ),
    tag = "threads",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_thread(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<UpdateThreadRequest>,
) -> Result<Json<ThreadResponse>, AppError> {
    // Validate input
    payload.validate()?;

    // Check if thread exists and user owns it
    let existing_thread = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM threads WHERE id = $1 AND user_id = $2)",
    )
    .bind(id)
    .bind(current_user.id)
    .fetch_one(&pool)
    .await?;

    if !existing_thread {
        return Err(AppError::NotFound);
    }

    if payload.title.is_none() && payload.content.is_none() {
        return Err(AppError::BadRequest("No fields to update".to_string()));
    }

    // Update thread with simplified query
    let _updated_thread = sqlx::query(
        r#"
        UPDATE threads 
        SET 
            title = COALESCE($2, title),
            content = COALESCE($3, content),
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(payload.title.as_ref())
    .bind(payload.content.as_ref())
    .execute(&pool)
    .await?;

    // Fetch user information and comment count
    let thread_with_user = sqlx::query_as::<_, ThreadWithUser>(
        r#"
        SELECT 
            t.id, t.title, t.content, t.created_at, t.updated_at,
            t.upvote_count, t.downvote_count,
            u.id as user_id, u.username, u.display_name as user_display_name, u.avatar_url as user_avatar_url,
            COUNT(c.id)::bigint as comment_count
        FROM threads t
        JOIN users u ON t.user_id = u.id
        LEFT JOIN comments c ON t.id = c.thread_id
        WHERE t.id = $1
        GROUP BY t.id, t.upvote_count, t.downvote_count, u.id, u.username, u.display_name, u.avatar_url
        "#
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(ThreadResponse::from(thread_with_user)))
}
