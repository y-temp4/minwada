use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{error::AppError, models::common::ErrorResponse};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct PaginationParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CommentListItem {
    pub id: Uuid,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub author_id: Uuid,
    pub author_username: String,
    pub thread_id: Uuid,
    pub thread_title: String,
    pub parent_id: Option<Uuid>,
}

#[derive(serde::Deserialize)]
pub struct PathParams {
    user_id: Uuid,
}

/// ユーザーが投稿したコメントの一覧を取得します
#[utoipa::path(
    get,
    path = "/api/users/{user_id}/comments",
    params(
        ("user_id" = Uuid, Path, description = "ユーザーID"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "コメント一覧の取得に成功", body = Vec<CommentListItem>),
        (status = 404, description = "ユーザーが見つからない", body = ErrorResponse),
        (status = 500, description = "サーバーエラー", body = ErrorResponse)
    )
)]
pub async fn get_user_comments(
    State(pool): State<PgPool>,
    Path(PathParams { user_id }): Path<PathParams>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<CommentListItem>>, AppError> {
    // デフォルト値の設定
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);

    // ユーザーが存在するか確認
    let user_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as exists",
        user_id
    )
    .fetch_one(&pool)
    .await?
    .exists
    .unwrap_or(false);

    if !user_exists {
        return Err(AppError::NotFound);
    }

    // ユーザーIDに基づいてコメントを取得
    let comments = sqlx::query_as!(
        CommentListItem,
        r#"
        SELECT
            c.id,
            c.content,
            c.created_at as "created_at!",
            c.updated_at as "updated_at!",
            c.user_id as "author_id!",
            u.username as "author_username!",
            c.thread_id as "thread_id!",
            t.title as "thread_title!",
            c.parent_id
        FROM
            comments c
        JOIN
            users u ON c.user_id = u.id
        JOIN
            threads t ON c.thread_id = t.id
        WHERE
            c.user_id = $1
        ORDER BY
            c.created_at DESC
        LIMIT $2
        OFFSET $3
        "#,
        user_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(comments))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_comment, create_test_thread, seed_test_user};
    use axum::extract::{Path, Query, State};
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_get_user_comments_success(pool: PgPool) -> Result<(), AppError> {
        // ユーザーが作成したコメントの一覧を正常に取得できるかテスト
        let user_id = seed_test_user(&pool, "comments_test1").await;

        // テストスレッドを作成
        let thread_id =
            create_test_thread(&pool, user_id, "Test Thread for Comments", "Test Content").await;

        // テストユーザーがコメントを3つ作成
        for i in 1..=3 {
            create_test_comment(
                &pool,
                user_id,
                thread_id,
                &format!("Test Comment {}", i),
                None,
            )
            .await;
        }

        // 別のユーザーを作成してコメントを作成（これは結果に含まれないはず）
        let other_user_id = seed_test_user(&pool, "comments_test2").await;
        create_test_comment(&pool, other_user_id, thread_id, "Other User Comment", None).await;

        // API呼び出し
        let result = get_user_comments(
            State(pool),
            Path(PathParams { user_id }),
            Query(PaginationParams {
                limit: Some(10),
                offset: Some(0),
            }),
        )
        .await?;

        let comments = result.0;

        // 正しい件数のコメントが取得できていることを確認
        assert_eq!(comments.len(), 3);
        // すべてのコメントが正しいユーザーに属していることを確認
        for comment in comments {
            assert_eq!(comment.author_id, user_id);
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_user_comments_pagination(pool: PgPool) -> Result<(), AppError> {
        // ページネーションが正しく機能するかテスト
        let user_id = seed_test_user(&pool, "comments_pagination_test").await;

        // テストスレッドを作成
        let thread_id =
            create_test_thread(&pool, user_id, "Test Thread for Pagination", "Test Content").await;

        // テストユーザーがコメントを5つ作成
        for i in 1..=5 {
            create_test_comment(
                &pool,
                user_id,
                thread_id,
                &format!("Test Comment {}", i),
                None,
            )
            .await;
        }

        // 最初の2つを取得
        let result1 = get_user_comments(
            State(pool.clone()),
            Path(PathParams { user_id }),
            Query(PaginationParams {
                limit: Some(2),
                offset: Some(0),
            }),
        )
        .await?;

        // 次の2つを取得
        let result2 = get_user_comments(
            State(pool.clone()),
            Path(PathParams { user_id }),
            Query(PaginationParams {
                limit: Some(2),
                offset: Some(2),
            }),
        )
        .await?;

        let comments1 = result1.0;
        let comments2 = result2.0;

        // 正しい件数のコメントが取得できていることを確認
        assert_eq!(comments1.len(), 2);
        assert_eq!(comments2.len(), 2);

        // IDが重複していないことを確認（異なるページのデータである）
        let ids1: Vec<Uuid> = comments1.iter().map(|t| t.id).collect();
        let ids2: Vec<Uuid> = comments2.iter().map(|t| t.id).collect();

        for id in &ids1 {
            assert!(!ids2.contains(id));
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_user_comments_empty(pool: PgPool) -> Result<(), AppError> {
        // ユーザーがコメントを作成していない場合、空の配列が返されることをテスト
        let user_id = seed_test_user(&pool, "comments_empty_test").await;

        // API呼び出し
        let result = get_user_comments(
            State(pool),
            Path(PathParams { user_id }),
            Query(PaginationParams {
                limit: Some(10),
                offset: Some(0),
            }),
        )
        .await?;

        let comments = result.0;

        // 空の配列が返されることを確認
        assert_eq!(comments.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_user_comments_user_not_found(pool: PgPool) {
        // 存在しないユーザーIDでリクエストした場合にエラーが返されることをテスト
        let non_existent_user_id = Uuid::new_v4();

        // API呼び出し
        let result = get_user_comments(
            State(pool),
            Path(PathParams {
                user_id: non_existent_user_id,
            }),
            Query(PaginationParams {
                limit: Some(10),
                offset: Some(0),
            }),
        )
        .await;

        // NotFoundエラーが返されることを確認
        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
