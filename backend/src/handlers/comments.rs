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
        comments::{
            CommentListResponse, CommentResponse, CommentWithUser, CreateCommentRequest,
            UpdateCommentRequest,
        },
        common::ErrorResponse,
        User,
    },
};

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

#[utoipa::path(
    delete,
    path = "/api/comments/{id}",
    params(
        ("id" = Uuid, Path, description = "Comment ID")
    ),
    responses(
        (status = 204, description = "Comment deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Comment not found", body = ErrorResponse)
    ),
    tag = "comments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_comment(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<User>,
) -> Result<StatusCode, AppError> {
    // Check if comment exists and user owns it, then delete
    let deleted_rows = sqlx::query("DELETE FROM comments WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(current_user.id)
        .execute(&pool)
        .await?;

    if deleted_rows.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

fn build_comment_tree(comments: Vec<CommentWithUser>) -> Vec<CommentResponse> {
    // 1. 最初にすべてのコメントをCommentResponseに変換
    let mut all_comments: Vec<CommentResponse> =
        comments.into_iter().map(|c| c.to_response()).collect();

    // 2. created_at順でソート（決定的な順序を保証）
    all_comments.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    // 3. ルートコメント（parent_id = None）と子コメントを分離
    let mut root_comments: Vec<CommentResponse> = Vec::new();
    let mut child_comments: Vec<CommentResponse> = Vec::new();

    for comment in all_comments {
        if comment.parent_id.is_none() {
            root_comments.push(comment);
        } else {
            child_comments.push(comment);
        }
    }

    // 4. 子コメントをルートコメントの適切な位置に配置
    fn add_children_to_parent(parent: &mut CommentResponse, children: &[CommentResponse]) {
        let mut direct_children: Vec<CommentResponse> = children
            .iter()
            .filter(|child| child.parent_id == Some(parent.id))
            .cloned()
            .collect();

        // 子コメントもcreated_at順でソート
        direct_children.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        for mut child in direct_children {
            // 再帰的に孫コメントも追加
            add_children_to_parent(&mut child, children);
            parent.replies.push(child);
            parent.reply_count += 1;
        }
    }

    // 5. 各ルートコメントに子コメントを追加
    for root_comment in &mut root_comments {
        add_children_to_parent(root_comment, &child_comments);
    }

    root_comments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;
    use axum::http::StatusCode;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_create_comment_success(pool: PgPool) {
        // テスト：メール認証済みユーザーがコメントを正常に作成できる
        let user = test_utils::create_test_user(&pool, true).await;
        let thread_id =
            test_utils::create_test_thread(&pool, user.id, "Test Thread", "Content").await;

        let request = CreateCommentRequest {
            content: "This is a test comment".to_string(),
            parent_id: None,
        };

        let result =
            create_comment(State(pool), Path(thread_id), Extension(user), Json(request)).await;

        assert!(result.is_ok());
        let (status, _) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
    }

    #[sqlx::test]
    async fn test_create_comment_email_not_verified(pool: PgPool) {
        // テスト：メール認証していないユーザーがコメント作成を試みるとエラーになる
        let user = test_utils::create_test_user(&pool, false).await;
        let thread_id =
            test_utils::create_test_thread(&pool, user.id, "Test Thread", "Content").await;

        let request = CreateCommentRequest {
            content: "This is a test comment".to_string(),
            parent_id: None,
        };

        let result =
            create_comment(State(pool), Path(thread_id), Extension(user), Json(request)).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::EmailVerificationRequired => (),
            err => panic!("Expected EmailVerificationRequired, got {:?}", err),
        }
    }
}
