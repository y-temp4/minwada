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
pub struct ThreadListItem {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub author_id: Uuid,
    pub author_username: String,
    pub comment_count: i64,
}

#[derive(serde::Deserialize)]
pub struct PathParams {
    user_id: Uuid,
}

/// ユーザーが投稿したスレッドの一覧を取得します
#[utoipa::path(
    get,
    path = "/api/users/{user_id}/threads",
    params(
        ("user_id" = Uuid, Path, description = "ユーザーID"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "スレッド一覧の取得に成功", body = Vec<ThreadListItem>),
        (status = 404, description = "ユーザーが見つからない", body = ErrorResponse),
        (status = 500, description = "サーバーエラー", body = ErrorResponse)
    )
)]
pub async fn get_user_threads(
    State(pool): State<PgPool>,
    Path(PathParams { user_id }): Path<PathParams>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<ThreadListItem>>, AppError> {
    // デフォルト値の設定
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);

    // ユーザーIDに基づいてスレッドを取得
    let threads = sqlx::query_as!(
        ThreadListItem,
        r#"
        SELECT
            t.id,
            t.title,
            t.content,
            t.created_at as "created_at!",
            t.updated_at as "updated_at!",
            t.user_id as "author_id!",
            u.username as "author_username!",
            COALESCE(COUNT(c.id), 0)::bigint as "comment_count!"
        FROM
            threads t
        JOIN
            users u ON t.user_id = u.id
        LEFT JOIN
            comments c ON c.thread_id = t.id
        WHERE
            t.user_id = $1
        GROUP BY
            t.id, u.username
        ORDER BY
            t.created_at DESC
        LIMIT $2
        OFFSET $3
        "#,
        user_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(threads))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_thread, seed_test_user};
    use axum::extract::{Path, Query, State};
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_get_user_threads_success(pool: PgPool) -> Result<(), AppError> {
        // ユーザーが作成したスレッドの一覧を正常に取得できるかテスト
        let user_id = seed_test_user(&pool, "threads_test1").await;

        // テストユーザーがスレッドを3つ作成
        for i in 1..=3 {
            create_test_thread(
                &pool,
                user_id,
                &format!("Test Thread {}", i),
                &format!("Test Content {}", i),
            )
            .await;
        }

        // 別のユーザーを作成してスレッドを作成（これは結果に含まれないはず）
        let other_user_id = seed_test_user(&pool, "threads_test2").await;
        create_test_thread(
            &pool,
            other_user_id,
            "Other User Thread",
            "Other User Content",
        )
        .await;

        // API呼び出し
        let result = get_user_threads(
            State(pool),
            Path(PathParams { user_id }),
            Query(PaginationParams {
                limit: Some(10),
                offset: Some(0),
            }),
        )
        .await?;

        let threads = result.0;

        // 正しい件数のスレッドが取得できていることを確認
        assert_eq!(threads.len(), 3);
        // すべてのスレッドが正しいユーザーに属していることを確認
        for thread in threads {
            assert_eq!(thread.author_id, user_id);
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_user_threads_pagination(pool: PgPool) -> Result<(), AppError> {
        // ページネーションが正しく機能するかテスト
        let user_id = seed_test_user(&pool, "pagination_test").await;

        // テストユーザーがスレッドを5つ作成
        for i in 1..=5 {
            create_test_thread(
                &pool,
                user_id,
                &format!("Test Thread {}", i),
                &format!("Test Content {}", i),
            )
            .await;
        }

        // 最初の2つを取得
        let result1 = get_user_threads(
            State(pool.clone()),
            Path(PathParams { user_id }),
            Query(PaginationParams {
                limit: Some(2),
                offset: Some(0),
            }),
        )
        .await?;

        // 次の2つを取得
        let result2 = get_user_threads(
            State(pool.clone()),
            Path(PathParams { user_id }),
            Query(PaginationParams {
                limit: Some(2),
                offset: Some(2),
            }),
        )
        .await?;

        let threads1 = result1.0;
        let threads2 = result2.0;

        // 正しい件数のスレッドが取得できていることを確認
        assert_eq!(threads1.len(), 2);
        assert_eq!(threads2.len(), 2);

        // IDが重複していないことを確認（異なるページのデータである）
        let ids1: Vec<Uuid> = threads1.iter().map(|t| t.id).collect();
        let ids2: Vec<Uuid> = threads2.iter().map(|t| t.id).collect();

        for id in &ids1 {
            assert!(!ids2.contains(id));
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_user_threads_empty(pool: PgPool) -> Result<(), AppError> {
        // ユーザーがスレッドを作成していない場合、空の配列が返されることをテスト
        let user_id = seed_test_user(&pool, "empty_test").await;

        // API呼び出し
        let result = get_user_threads(
            State(pool),
            Path(PathParams { user_id }),
            Query(PaginationParams {
                limit: Some(10),
                offset: Some(0),
            }),
        )
        .await?;

        let threads = result.0;

        // 空の配列が返されることを確認
        assert_eq!(threads.len(), 0);

        Ok(())
    }
}
