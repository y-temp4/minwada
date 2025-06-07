use axum::{extract::Path, extract::State, Json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        comments::{CommentListResponse, CommentWithUser},
        common::ErrorResponse,
    },
};

use super::utils::build_comment_tree;

#[utoipa::path(
    get,
    path = "/api/threads/{thread_id}/comments",
    params(
        ("thread_id" = Uuid, Path, description = "Thread ID")
    ),
    responses(
        (status = 200, description = "List of comments", body = CommentListResponse),
        (status = 404, description = "Thread not found", body = ErrorResponse)
    ),
    tag = "comments"
)]
pub async fn get_comments(
    State(pool): State<PgPool>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<CommentListResponse>, AppError> {
    // Check if thread exists
    let thread_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM threads WHERE id = $1)")
            .bind(thread_id)
            .fetch_one(&pool)
            .await?;

    if !thread_exists {
        return Err(AppError::NotFound);
    }

    // Get all comments for the thread with user information
    let comments_with_users = sqlx::query_as::<_, CommentWithUser>(
        r#"
        SELECT 
            c.id, c.thread_id, c.content, c.parent_id, c.created_at, c.updated_at,
            u.id as user_id, u.username, u.display_name as user_display_name, u.avatar_url as user_avatar_url
        FROM comments c
        JOIN users u ON c.user_id = u.id
        WHERE c.thread_id = $1
        ORDER BY c.created_at ASC
        "#
    )
    .bind(thread_id)
    .fetch_all(&pool)
    .await?;

    // Store the total count before building tree structure
    let total_count = comments_with_users.len() as u64;

    // Build tree structure
    let comment_tree = build_comment_tree(comments_with_users);

    // Debug: Print comment tree structure
    println!("comment_tree: {:?}", comment_tree);

    Ok(Json(CommentListResponse {
        comments: comment_tree,
        total_count,
    }))
}