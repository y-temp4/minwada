use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::PgPool;

use super::models::ThreadQuery;
use crate::{
    error::AppError,
    models::{
        common::PaginatedResponse,
        threads::{ThreadListResponse, ThreadResponse, ThreadWithUser},
    },
};

#[utoipa::path(
    get,
    path = "/api/threads",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<u32>, Query, description = "Number of items per page (default: 20)")
    ),
    responses(
        (status = 200, description = "List of threads", body = ThreadListResponse)
    ),
    tag = "threads"
)]
pub async fn get_threads(
    State(pool): State<PgPool>,
    Query(query): Query<ThreadQuery>,
) -> Result<Json<ThreadListResponse>, AppError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    // Get total count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM threads")
        .fetch_one(&pool)
        .await?;

    // Get threads with user information and comment count
    let threads = sqlx::query_as::<_, ThreadWithUser>(
        r#"
        SELECT 
            t.id, t.title, t.content, t.created_at, t.updated_at,
            u.id as user_id, u.username, u.display_name as user_display_name, u.avatar_url as user_avatar_url,
            COUNT(c.id)::bigint as comment_count
        FROM threads t
        JOIN users u ON t.user_id = u.id
        LEFT JOIN comments c ON t.id = c.thread_id
        GROUP BY t.id, u.id, u.username, u.display_name, u.avatar_url
        ORDER BY t.created_at DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&pool)
    .await?;

    let thread_responses: Vec<ThreadResponse> =
        threads.into_iter().map(ThreadResponse::from).collect();

    let paginated_response = PaginatedResponse::new(thread_responses, total as u64, page, limit);

    Ok(Json(ThreadListResponse {
        threads: paginated_response,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::threads::test_utils::{
        cleanup_test_data, create_second_thread, seed_test_data, setup_test_db,
    };

    #[tokio::test]
    async fn test_get_threads() {
        // テスト用データベースをセットアップ
        let pool = setup_test_db().await;

        // テストデータを準備（ユニークな識別子を使用）
        let (user_id, thread_id) = seed_test_data(&pool, "threads_test").await;

        // テスト実行: スレッド一覧を取得
        let query = ThreadQuery {
            page: Some(1),
            limit: Some(10),
        };

        let result = get_threads(State(pool.clone()), Query(query)).await;

        // アサーション
        assert!(result.is_ok(), "get_threads should return Ok");
        let response = result.unwrap();

        // レスポンスの内容を確認
        let threads = &response.0.threads;
        assert!(threads.total > 0, "Should have at least one thread");
        assert_eq!(threads.page, 1, "Page number should be 1");
        assert_eq!(threads.limit, 10, "Limit should be 10");

        // テストデータのクリーンアップ
        cleanup_test_data(&pool, thread_id, user_id).await;
    }

    #[tokio::test]
    async fn test_get_threads_pagination() {
        // テスト用データベースをセットアップ
        let pool = setup_test_db().await;

        // 複数のテストデータを準備（ユニークな識別子を使用）
        let (user_id1, thread_id1) = seed_test_data(&pool, "pagination_test").await;

        // 2つ目のテストスレッドを作成
        let thread_id2 = create_second_thread(
            &pool,
            user_id1,
            "Second Test Thread",
            "This is another test thread content",
        )
        .await;

        // ページングテスト: リミット1で1ページ目を取得
        let query1 = ThreadQuery {
            page: Some(1),
            limit: Some(1),
        };
        let result1 = get_threads(State(pool.clone()), Query(query1))
            .await
            .unwrap();

        // ページングテスト: リミット1で2ページ目を取得
        let query2 = ThreadQuery {
            page: Some(2),
            limit: Some(1),
        };
        let result2 = get_threads(State(pool.clone()), Query(query2))
            .await
            .unwrap();

        // アサーション
        assert_eq!(
            result1.0.threads.data.len(),
            1,
            "First page should have exactly 1 thread"
        );
        assert_eq!(
            result2.0.threads.data.len(),
            1,
            "Second page should have exactly 1 thread"
        );
        assert_ne!(
            result1.0.threads.data[0].id, result2.0.threads.data[0].id,
            "Threads on different pages should be different"
        );

        // テストデータのクリーンアップ
        cleanup_test_data(&pool, thread_id1, user_id1).await;
        sqlx::query("DELETE FROM threads WHERE id = $1")
            .bind(thread_id2)
            .execute(&pool)
            .await
            .expect("Failed to delete second test thread");
    }
}
