use axum::{extract::Extension, extract::Path, extract::State, http::StatusCode};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, models::common::ErrorResponse, models::User};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_comment, create_test_user, seed_test_data};
    use axum::extract::{Extension, Path, State};

    #[sqlx::test]
    async fn test_コメント削除_成功(pool: PgPool) {
        // コメント作成者による削除が成功することを確認
        let (_user_id, thread_id) = seed_test_data(&pool, "comment_delete_success").await;
        let user = create_test_user(&pool, true).await;

        // Create a comment to delete
        let comment_id = create_test_comment(&pool, user.id, thread_id, "Test comment", None).await;

        let result = delete_comment(State(pool.clone()), Path(comment_id), Extension(user)).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);

        // Verify comment is deleted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE id = $1")
            .bind(comment_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count comments");
        assert_eq!(count, 0);
    }

    #[sqlx::test]
    async fn test_コメント削除_存在しないコメントエラー(pool: PgPool) {
        // 存在しないコメントの削除が失敗することを確認
        let user = create_test_user(&pool, true).await;
        let non_existent_comment_id = Uuid::new_v4();

        let result = delete_comment(
            State(pool.clone()),
            Path(non_existent_comment_id),
            Extension(user),
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
    async fn test_コメント削除_他人のコメント削除禁止エラー(pool: PgPool) {
        // 他人のコメントを削除できないことを確認
        let (user_id, thread_id) = seed_test_data(&pool, "comment_delete_forbidden").await;
        let other_user = create_test_user(&pool, true).await;

        // Create a comment by another user
        let comment_id =
            create_test_comment(&pool, user_id, thread_id, "Other user's comment", None).await;

        let result =
            delete_comment(State(pool.clone()), Path(comment_id), Extension(other_user)).await;

        assert!(result.is_err());
        if let Err(AppError::NotFound) = result {
            // Expected error (returns NotFound instead of Forbidden for security)
        } else {
            panic!("Expected NotFound error");
        }

        // Verify comment still exists
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE id = $1")
            .bind(comment_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count comments");
        assert_eq!(count, 1);
    }

    #[sqlx::test]
    async fn test_コメント削除_子コメント存在時も削除可能(pool: PgPool) {
        // 子コメントが存在する親コメントでも削除可能であることを確認
        let (user_id, thread_id) = seed_test_data(&pool, "comment_delete_with_children").await;
        let user = create_test_user(&pool, true).await;

        // Create parent comment
        let parent_comment_id =
            create_test_comment(&pool, user.id, thread_id, "Parent comment", None).await;

        // Create child comment
        let _child_comment_id = create_test_comment(
            &pool,
            user_id,
            thread_id,
            "Child comment",
            Some(parent_comment_id),
        )
        .await;

        let result = delete_comment(
            State(pool.clone()),
            Path(parent_comment_id),
            Extension(user),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);

        // Verify parent comment is deleted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE id = $1")
            .bind(parent_comment_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count comments");
        assert_eq!(count, 0);
    }
}
