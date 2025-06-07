use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        comments::{CommentResponse, CommentWithUser, CreateCommentRequest},
        common::ErrorResponse,
        User,
    },
};

#[utoipa::path(
    post,
    path = "/api/threads/{thread_id}/comments",
    params(
        ("thread_id" = Uuid, Path, description = "Thread ID")
    ),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created successfully", body = CommentResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Thread not found", body = ErrorResponse)
    ),
    tag = "comments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_comment(
    State(pool): State<PgPool>,
    Path(thread_id): Path<Uuid>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CommentResponse>), AppError> {
    // Validate input
    payload.validate()?;

    // メール認証が完了しているか確認
    if !current_user.email_verified {
        return Err(AppError::EmailVerificationRequired);
    }

    // Check if thread exists
    let thread_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM threads WHERE id = $1)")
            .bind(thread_id)
            .fetch_one(&pool)
            .await?;

    if !thread_exists {
        return Err(AppError::NotFound);
    }

    // If parent_id is provided, check if parent comment exists and belongs to the same thread
    if let Some(parent_id) = payload.parent_id {
        let parent_comment = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM comments WHERE id = $1 AND thread_id = $2)",
        )
        .bind(parent_id)
        .bind(thread_id)
        .fetch_one(&pool)
        .await?;

        if !parent_comment {
            return Err(AppError::BadRequest(
                "Parent comment not found or does not belong to this thread".to_string(),
            ));
        }
    }

    // Create comment
    let comment = sqlx::query_as::<_, CommentWithUser>(
        r#"
        INSERT INTO comments (thread_id, user_id, content, parent_id)
        VALUES ($1, $2, $3, $4)
        RETURNING 
            id, $1 as thread_id, content, parent_id, created_at, updated_at,
            $2 as user_id, $5 as username, $6 as user_display_name, $7 as user_avatar_url
        "#,
    )
    .bind(thread_id)
    .bind(current_user.id)
    .bind(&payload.content)
    .bind(payload.parent_id)
    .bind(&current_user.username)
    .bind(&current_user.display_name)
    .bind(&current_user.avatar_url)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(comment.to_response())))
}