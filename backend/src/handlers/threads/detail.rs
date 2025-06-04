use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::threads::{ThreadResponse, ThreadWithUser},
};

#[utoipa::path(
    get,
    path = "/api/threads/{id}",
    params(
        ("id" = Uuid, Path, description = "Thread ID")
    ),
    responses(
        (status = 200, description = "Thread details", body = ThreadResponse),
        (status = 404, description = "Thread not found", body = ErrorResponse)
    ),
    tag = "threads"
)]
pub async fn get_thread(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ThreadResponse>, AppError> {
    let thread = sqlx::query_as::<_, ThreadWithUser>(
        r#"
        SELECT 
            t.id, t.title, t.content, t.created_at, t.updated_at,
            u.id as user_id, u.username, u.display_name as user_display_name, u.avatar_url as user_avatar_url,
            COUNT(c.id)::bigint as comment_count
        FROM threads t
        JOIN users u ON t.user_id = u.id
        LEFT JOIN comments c ON t.id = c.thread_id
        WHERE t.id = $1
        GROUP BY t.id, u.id, u.username, u.display_name, u.avatar_url
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(ThreadResponse::from(thread)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::threads::test_utils::{cleanup_test_data, seed_test_data, setup_test_db};

    #[tokio::test]
    async fn test_get_thread_success() {
        // テスト用データベースをセットアップ
        let pool = setup_test_db().await;

        // テストデータを準備（ユニークな識別子を使用）
        let (user_id, thread_id) = seed_test_data(&pool, "detail_test").await;

        // テスト実行: 特定のスレッドを取得
        let result = get_thread(State(pool.clone()), Path(thread_id)).await;

        // アサーション
        assert!(result.is_ok(), "get_thread should return Ok");
        let response = result.unwrap();
        let thread = &response.0;

        assert_eq!(thread.id, thread_id, "Thread ID should match");
        assert_eq!(
            thread.title, "Test Thread detail_test",
            "Thread title should match"
        );
        assert!(thread.content.is_some(), "Thread content should exist");
        assert_eq!(thread.user.id, user_id, "User ID should match");

        // テストデータのクリーンアップ
        cleanup_test_data(&pool, thread_id, user_id).await;
    }

    #[tokio::test]
    async fn test_get_thread_not_found() {
        // テスト用データベースをセットアップ
        let pool = setup_test_db().await;

        // 存在しないUUIDを使用
        let non_existent_id = Uuid::new_v4();

        // テスト実行: 存在しないスレッドを取得
        let result = get_thread(State(pool.clone()), Path(non_existent_id)).await;

        // アサーション
        assert!(
            result.is_err(),
            "get_thread should return error for non-existent thread"
        );

        // エラーがNotFoundであることを確認
        match result {
            Err(AppError::NotFound) => (),
            _ => panic!("Expected NotFound error"),
        }
    }
}
