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

        // Check comment hierarchy depth (max 4 levels)
        let depth = calculate_comment_depth(&pool, parent_id).await?;
        if depth >= 4 {
            return Err(AppError::BadRequest(
                "Maximum comment nesting depth (4 levels) exceeded".to_string(),
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

async fn calculate_comment_depth(pool: &PgPool, comment_id: Uuid) -> Result<i32, AppError> {
    let depth = sqlx::query_scalar::<_, i32>(
        r#"
        WITH RECURSIVE comment_hierarchy AS (
            SELECT id, parent_id, 1 as depth
            FROM comments
            WHERE id = $1
            
            UNION ALL
            
            SELECT c.id, c.parent_id, ch.depth + 1
            FROM comments c
            INNER JOIN comment_hierarchy ch ON c.id = ch.parent_id
        )
        SELECT MAX(depth) FROM comment_hierarchy
        "#,
    )
    .bind(comment_id)
    .fetch_one(pool)
    .await?;

    Ok(depth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::comments::CreateCommentRequest,
        test_utils::{create_test_comment, create_test_user, seed_test_data},
    };
    use axum::{
        extract::{Extension, Path, State},
        http::StatusCode,
        Json,
    };

    #[sqlx::test]
    async fn test_コメント作成_成功(pool: PgPool) {
        // メール認証済みユーザーでコメント作成が成功することを確認
        let (user_id, thread_id) = seed_test_data(&pool, "comment_create_success").await;

        // Update user to be email verified
        sqlx::query(
            "UPDATE users SET email_verified = true, email_verified_at = NOW() WHERE id = $1",
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Failed to update user verification status");

        let verified_user =
            sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to fetch verified test user");

        let request = CreateCommentRequest {
            content: "Test comment content".to_string(),
            parent_id: None,
        };

        let result = create_comment(
            State(pool.clone()),
            Path(thread_id),
            Extension(verified_user),
            Json(request),
        )
        .await;

        assert!(result.is_ok());
        let (status, _) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
    }

    #[sqlx::test]
    async fn test_コメント作成_メール未認証エラー(pool: PgPool) {
        // メール未認証ユーザーによるコメント作成が失敗することを確認
        let (user_id, thread_id) = seed_test_data(&pool, "comment_unverified").await;
        let user = sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch test user");

        let request = CreateCommentRequest {
            content: "Test comment content".to_string(),
            parent_id: None,
        };

        let result = create_comment(
            State(pool.clone()),
            Path(thread_id),
            Extension(user),
            Json(request),
        )
        .await;

        assert!(result.is_err());
        if let Err(AppError::EmailVerificationRequired) = result {
            // Expected error
        } else {
            panic!("Expected EmailVerificationRequired error");
        }
    }

    #[sqlx::test]
    async fn test_コメント作成_スレッド存在しないエラー(pool: PgPool) {
        // 存在しないスレッドに対するコメント作成が失敗することを確認
        let user = create_test_user(&pool, true).await;
        let non_existent_thread_id = Uuid::new_v4();

        let request = CreateCommentRequest {
            content: "Test comment content".to_string(),
            parent_id: None,
        };

        let result = create_comment(
            State(pool.clone()),
            Path(non_existent_thread_id),
            Extension(user),
            Json(request),
        )
        .await;

        assert!(result.is_err());
        if let Err(AppError::NotFound) = result {
            // Expected error
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[sqlx::test]
    async fn test_コメント作成_親コメント存在しないエラー(pool: PgPool) {
        // 存在しない親コメントに対するリプライが失敗することを確認
        let (_user_id, thread_id) = seed_test_data(&pool, "comment_parent_not_found").await;
        let user = create_test_user(&pool, true).await;
        let non_existent_parent_id = Uuid::new_v4();

        let request = CreateCommentRequest {
            content: "Test comment content".to_string(),
            parent_id: Some(non_existent_parent_id),
        };

        let result = create_comment(
            State(pool.clone()),
            Path(thread_id),
            Extension(user),
            Json(request),
        )
        .await;

        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("Parent comment not found"));
        } else {
            panic!("Expected BadRequest error about parent comment not found");
        }
    }

    #[sqlx::test]
    async fn test_コメント作成_リプライ成功(pool: PgPool) {
        // 親コメントに対するリプライが成功することを確認
        let (user_id, thread_id) = seed_test_data(&pool, "comment_reply_success").await;
        let user = create_test_user(&pool, true).await;

        // Create a parent comment
        let parent_comment_id =
            create_test_comment(&pool, user_id, thread_id, "Parent comment", None).await;

        let request = CreateCommentRequest {
            content: "Reply comment content".to_string(),
            parent_id: Some(parent_comment_id),
        };

        let result = create_comment(
            State(pool.clone()),
            Path(thread_id),
            Extension(user),
            Json(request),
        )
        .await;

        assert!(result.is_ok());
        let (status, _) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
    }

    #[sqlx::test]
    async fn test_コメント作成_最大階層深度超過エラー(pool: PgPool) {
        // 最大階層深度（4階層）を超えるコメント作成が失敗することを確認
        let (user_id, thread_id) = seed_test_data(&pool, "comment_max_depth").await;
        let user = create_test_user(&pool, true).await;

        // Create a deep comment hierarchy (4 levels)
        let comment1 = create_test_comment(&pool, user_id, thread_id, "Level 1", None).await;
        let comment2 =
            create_test_comment(&pool, user_id, thread_id, "Level 2", Some(comment1)).await;
        let comment3 =
            create_test_comment(&pool, user_id, thread_id, "Level 3", Some(comment2)).await;
        let comment4 =
            create_test_comment(&pool, user_id, thread_id, "Level 4", Some(comment3)).await;

        // Try to create a 5th level comment (should fail)
        let request = CreateCommentRequest {
            content: "Level 5 comment (should fail)".to_string(),
            parent_id: Some(comment4),
        };

        let result = create_comment(
            State(pool.clone()),
            Path(thread_id),
            Extension(user),
            Json(request),
        )
        .await;

        assert!(result.is_err());
        if let Err(AppError::BadRequest(msg)) = result {
            assert!(msg.contains("Maximum comment nesting depth"));
        } else {
            panic!("Expected BadRequest error about maximum depth exceeded");
        }
    }

    #[sqlx::test]
    async fn test_コメント階層深度計算(pool: PgPool) {
        // コメント階層深度の計算が正しく動作することを確認
        let (user_id, thread_id) = seed_test_data(&pool, "calculate_depth").await;

        // Create a comment hierarchy
        let comment1 = create_test_comment(&pool, user_id, thread_id, "Level 1", None).await;
        let comment2 =
            create_test_comment(&pool, user_id, thread_id, "Level 2", Some(comment1)).await;
        let comment3 =
            create_test_comment(&pool, user_id, thread_id, "Level 3", Some(comment2)).await;

        // Test depth calculation
        let depth1 = calculate_comment_depth(&pool, comment1).await.unwrap();
        let depth2 = calculate_comment_depth(&pool, comment2).await.unwrap();
        let depth3 = calculate_comment_depth(&pool, comment3).await.unwrap();

        assert_eq!(depth1, 1);
        assert_eq!(depth2, 2);
        assert_eq!(depth3, 3);
    }
}
