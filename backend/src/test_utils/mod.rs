#[cfg(test)]
use crate::models::User;
#[cfg(test)]
use chrono::Utc;
#[cfg(test)]
use sqlx::PgPool;
#[cfg(test)]
use uuid::Uuid;

// テスト用の完全なユーザーオブジェクトを作成して返す関数
#[cfg(test)]
pub async fn create_test_user(pool: &PgPool, email_verified: bool) -> crate::models::User {
    let user_id = Uuid::new_v4();
    let username = format!(
        "testuser_{}",
        user_id.to_string().split('-').next().unwrap()
    );
    let email = format!("test_{}@example.com", username);
    let now = Utc::now();

    // テスト用ユーザーを作成
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, display_name, avatar_url, email_verified, email_verified_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(user_id)
    .bind(&username)
    .bind(&email)
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(email_verified)
    .bind(if email_verified { Some(now) } else { None })
    .bind(now)
    .bind(now)
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
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .expect("Failed to create test user credentials");

    // ユーザーオブジェクトを取得して返す
    sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("Failed to fetch created test user")
}

// テスト用のユーザーデータを作成する関数
#[cfg(test)]
pub async fn seed_test_user(pool: &PgPool, username_suffix: &str) -> Uuid {
    // テスト用ユーザーを作成（ユニークなユーザー名）
    let user_id = Uuid::new_v4();
    let username = format!("testuser_{}", username_suffix);
    let email = format!("test_{}@example.com", username_suffix);

    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, display_name, avatar_url, email_verified, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(user_id)
    .bind(&username)
    .bind(&email)
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(false)
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

    user_id
}

// テスト用のスレッドを作成する関数
#[cfg(test)]
pub async fn create_test_thread(pool: &PgPool, user_id: Uuid, title: &str, content: &str) -> Uuid {
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

// テスト用のコメントを作成する関数
#[cfg(test)]
pub async fn create_test_comment(
    pool: &PgPool,
    user_id: Uuid,
    thread_id: Uuid,
    content: &str,
    parent_id: Option<Uuid>,
) -> Uuid {
    let comment_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO comments (id, thread_id, user_id, parent_id, content, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(comment_id)
    .bind(thread_id)
    .bind(user_id)
    .bind(parent_id)
    .bind(content)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(pool)
    .await
    .expect("Failed to create comment");

    comment_id
}

// テスト用のユーザーとスレッドデータを作成する関数
#[cfg(test)]
pub async fn seed_test_data(pool: &PgPool, username_suffix: &str) -> (Uuid, Uuid) {
    // ユーザーを作成
    let user_id = seed_test_user(pool, username_suffix).await;

    // デフォルトのテストスレッドタイトルとコンテンツ
    let title = format!("Test Thread {}", username_suffix);
    let content = format!("This is a test thread content {}", username_suffix);

    // スレッドを作成
    let thread_id = create_test_thread(pool, user_id, &title, &content).await;

    (user_id, thread_id)
}
