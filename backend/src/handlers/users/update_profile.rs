use axum::{
    extract::{Extension, State},
    response::Json,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        common::ErrorResponse,
        users::{UpdateProfileRequest, UserResponse},
        User,
    },
};

#[utoipa::path(
    put,
    path = "/api/users/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Username already exists", body = ErrorResponse)
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_profile(
    State(pool): State<PgPool>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Validate input
    payload.validate()?;

    // Check if username is taken by another user
    if let Some(ref username) = payload.username {
        let existing_user = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username = $1 AND id != $2",
        )
        .bind(username)
        .bind(current_user.id)
        .fetch_one(&pool)
        .await?;

        if existing_user > 0 {
            return Err(AppError::Conflict("Username already exists".to_string()));
        }
    }

    if payload.username.is_none() && payload.display_name.is_none() && payload.avatar_url.is_none()
    {
        return Err(AppError::BadRequest("No fields to update".to_string()));
    }

    // Update user with simplified query
    let updated_user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users 
        SET 
            username = COALESCE($2, username),
            display_name = COALESCE($3, display_name),
            avatar_url = COALESCE($4, avatar_url),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(current_user.id)
    .bind(payload.username.as_ref())
    .bind(payload.display_name.as_ref())
    .bind(payload.avatar_url.as_ref())
    .fetch_one(&pool)
    .await?;

    Ok(Json(UserResponse::from(updated_user)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;

    use crate::test_utils::seed_test_user;

    #[sqlx::test]
    async fn test_update_profile_success(pool: PgPool) {
        // テストユーザーを作成
        let user_id = seed_test_user(&pool, "profile_update_test").await;

        // テスト用のユーザーを取得
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get test user");

        // 更新リクエストを作成
        let update_request = UpdateProfileRequest {
            username: Some("updated_username".to_string()),
            display_name: Some("Updated Display Name".to_string()),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
        };

        // ハンドラを直接呼び出し
        let result =
            update_profile(State(pool.clone()), Extension(user), Json(update_request)).await;

        // レスポンスを検証
        assert!(result.is_ok(), "update_profile should return Ok");

        // 更新されたユーザーを取得して検証
        let updated_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get updated user");

        assert_eq!(updated_user.username, "updated_username");
        assert_eq!(
            updated_user.display_name,
            Some("Updated Display Name".to_string())
        );
        assert_eq!(
            updated_user.avatar_url,
            Some("https://example.com/avatar.png".to_string())
        );
    }

    #[sqlx::test]
    async fn test_update_profile_username_exists(pool: PgPool) {
        // 2人のテストユーザーを作成
        let user1_id = seed_test_user(&pool, "profile_update_user1").await;
        seed_test_user(&pool, "profile_update_user2").await;

        // テスト用のユーザーを取得
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user1_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get test user");

        // 既存のユーザー名に更新しようとする
        let existing_username = format!("testuser_profile_update_user2");
        let update_request = UpdateProfileRequest {
            username: Some(existing_username),
            display_name: None,
            avatar_url: None,
        };

        // ハンドラを直接呼び出し
        let result =
            update_profile(State(pool.clone()), Extension(user), Json(update_request)).await;

        // エラーが返されることを確認
        assert!(
            result.is_err(),
            "Should return error for duplicate username"
        );

        // 競合エラーを確認
        match result {
            Err(AppError::Conflict(_)) => {} // 期待通り
            _ => panic!("Expected Conflict error"),
        }
    }

    #[sqlx::test]
    async fn test_update_profile_no_fields(pool: PgPool) {
        // テストユーザーを作成
        let user_id = seed_test_user(&pool, "profile_update_empty").await;

        // テスト用のユーザーを取得
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get test user");

        // 空のリクエストを作成
        let update_request = UpdateProfileRequest {
            username: None,
            display_name: None,
            avatar_url: None,
        };

        // ハンドラを直接呼び出し
        let result =
            update_profile(State(pool.clone()), Extension(user), Json(update_request)).await;

        // エラーが返されることを確認
        assert!(result.is_err(), "Should return error for empty update");

        // Bad Requestエラーを確認
        match result {
            Err(AppError::BadRequest(_)) => {} // 期待通り
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[sqlx::test]
    async fn test_update_profile_invalid_url(pool: PgPool) {
        // テストユーザーを作成
        let user_id = seed_test_user(&pool, "profile_update_invalid").await;

        // テスト用のユーザーを取得
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get test user");

        // 無効なURLでリクエストを作成
        let update_request = UpdateProfileRequest {
            username: None,
            display_name: None,
            avatar_url: Some("invalid-url".to_string()),
        };

        // ハンドラを直接呼び出し
        let result =
            update_profile(State(pool.clone()), Extension(user), Json(update_request)).await;

        // エラーが返されることを確認
        assert!(result.is_err(), "Should return error for invalid URL");

        // バリデーションエラーを確認
        match result {
            Err(AppError::Validation(_)) => {} // 期待通り
            _ => panic!("Expected Validation error"),
        }
    }

    #[sqlx::test]
    async fn test_update_profile_invalid_username(pool: PgPool) {
        // テストユーザーを作成
        let user_id = seed_test_user(&pool, "profile_update_invalid_username").await;

        // テスト用のユーザーを取得
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get test user");

        // 4文字未満の不正なユーザー名でリクエストを作成
        let update_request = UpdateProfileRequest {
            username: Some("abc".to_string()), // 3文字のユーザー名（最小は4文字必要）
            display_name: None,
            avatar_url: None,
        };

        // ハンドラを直接呼び出し
        let result =
            update_profile(State(pool.clone()), Extension(user), Json(update_request)).await;

        // エラーが返されることを確認
        assert!(result.is_err(), "Should return error for invalid username");

        // バリデーションエラーを確認
        match result {
            Err(AppError::Validation(_)) => {} // 期待通り
            _ => panic!("Expected Validation error"),
        }
    }

    #[sqlx::test]
    async fn test_update_profile_username_too_long(pool: PgPool) {
        // テストユーザーを作成
        let user_id = seed_test_user(&pool, "profile_update_username_too_long").await;

        // テスト用のユーザーを取得
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get test user");

        // 51文字の長すぎるユーザー名を作成（最大は30文字）
        let long_username = "a".repeat(31);
        let update_request = UpdateProfileRequest {
            username: Some(long_username),
            display_name: None,
            avatar_url: None,
        };

        // ハンドラを直接呼び出し
        let result =
            update_profile(State(pool.clone()), Extension(user), Json(update_request)).await;

        // エラーが返されることを確認
        assert!(result.is_err(), "Should return error for username too long");

        // バリデーションエラーを確認
        match result {
            Err(AppError::Validation(_)) => {} // 期待通り
            _ => panic!("Expected Validation error"),
        }
    }
}
