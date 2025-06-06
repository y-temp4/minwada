#[cfg(test)]
use sqlx::PgPool;
#[cfg(test)]
use uuid::Uuid;

// 共通テストユーティリティを再エクスポート
#[cfg(test)]
pub use crate::test_utils::{
    cleanup_test_data, create_test_thread, seed_test_data, setup_test_db
};

// 2つ目のテストスレッドを作成する関数（create_test_threadを使用）
#[cfg(test)]
pub async fn create_second_thread(
    pool: &PgPool,
    user_id: Uuid,
    title: &str,
    content: &str,
) -> Uuid {
    create_test_thread(pool, user_id, title, content).await
}
