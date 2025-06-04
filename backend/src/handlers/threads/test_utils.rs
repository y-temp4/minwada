#[cfg(test)]
use chrono::Utc;
#[cfg(test)]
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};
#[cfg(test)]
use uuid::Uuid;

// テスト用のデータベース設定
#[cfg(test)]
pub async fn setup_test_db() -> Pool<Postgres> {
    // テスト用の.envファイルの設定を読み込む
    dotenvy::dotenv().ok();

    // データベース接続を取得
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // テスト用のPostgreSQLデータベースに接続
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres")
}

// テスト用のスレッドデータを作成する関数
#[cfg(test)]
pub async fn seed_test_data(pool: &PgPool, username_suffix: &str) -> (Uuid, Uuid) {
    // テスト用ユーザーを作成（ユニークなユーザー名）
    let user_id = Uuid::new_v4();
    let username = format!("testuser_{}", username_suffix);
    let email = format!("test_{}@example.com", username_suffix);

    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, display_name, avatar_url, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(user_id)
    .bind(&username)
    .bind(&email)
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(pool)
    .await
    .expect("Failed to create test user");

    // テスト用の認証情報を作成
    sqlx::query(
        r#"
        INSERT INTO user_credentials (user_id, password_hash, salt, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(user_id)
    .bind("password_hash_value")
    .bind("salt_value")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(pool)
    .await
    .expect("Failed to create test user credentials");

    // テスト用のスレッドを作成
    let thread_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO threads (id, title, content, user_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(thread_id)
    .bind(format!("Test Thread {}", username_suffix))
    .bind(Some(format!(
        "This is a test thread content {}",
        username_suffix
    )))
    .bind(user_id)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(pool)
    .await
    .expect("Failed to create test thread");

    (user_id, thread_id)
}

// テスト終了後にテストデータをクリーンアップする関数
#[cfg(test)]
pub async fn cleanup_test_data(pool: &PgPool, thread_id: Uuid, user_id: Uuid) {
    // スレッドを削除
    sqlx::query("DELETE FROM threads WHERE id = $1")
        .bind(thread_id)
        .execute(pool)
        .await
        .expect("Failed to delete test thread");

    // ユーザーを削除
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .expect("Failed to delete test user");
}

// 2つ目のテストスレッドを作成する関数
#[cfg(test)]
pub async fn create_second_thread(
    pool: &PgPool,
    user_id: Uuid,
    title: &str,
    content: &str,
) -> Uuid {
    let thread_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO threads (id, title, content, user_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(thread_id)
    .bind(title)
    .bind(Some(content))
    .bind(user_id)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(pool)
    .await
    .expect("Failed to create thread");

    thread_id
}
