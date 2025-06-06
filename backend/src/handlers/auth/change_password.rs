use axum::{
    extract::{Extension, State},
    Json,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    auth::password::{hash_password, verify_password},
    error::AppError,
    models::{
        auth::{ChangePasswordRequest, MessageResponse},
        common::ErrorResponse,
        User, UserCredentials,
    },
};

#[utoipa::path(
    post,
    path = "/api/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = MessageResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Current password is incorrect", body = ErrorResponse)
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn change_password(
    State(pool): State<PgPool>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    // バリデーション実行
    payload.validate()?;

    // 現在のユーザーの認証情報を取得
    let credentials =
        sqlx::query_as::<_, UserCredentials>("SELECT * FROM user_credentials WHERE user_id = $1")
            .bind(current_user.id)
            .fetch_one(&pool)
            .await?;

    // 現在のパスワードを検証
    if !verify_password(&payload.current_password, &credentials.password_hash)? {
        return Err(AppError::Forbidden);
    }

    // 新しいパスワードをハッシュ化
    let (password_hash, salt) = hash_password(&payload.new_password)?;

    // パスワードを更新
    sqlx::query(
        r#"
        UPDATE user_credentials 
        SET password_hash = $1, salt = $2, updated_at = NOW() 
        WHERE user_id = $3
        "#,
    )
    .bind(&password_hash)
    .bind(&salt)
    .bind(current_user.id)
    .execute(&pool)
    .await?;

    Ok(Json(MessageResponse {
        message: "パスワードが正常に変更されました".to_string(),
    }))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{auth::password::hash_password, test_utils::seed_test_user};

    #[sqlx::test]
    async fn test_change_password_success(pool: PgPool) {
        // テストユーザーを作成
        let user_id = seed_test_user(&pool, "change_password_test").await;

        // テスト用のユーザーを取得
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get test user");

        // 既存のパスワードをセットアップ
        let current_password = "password123";
        let (password_hash, salt) = hash_password(current_password).unwrap();

        // ユーザー認証情報を更新
        sqlx::query("UPDATE user_credentials SET password_hash = $1, salt = $2 WHERE user_id = $3")
            .bind(&password_hash)
            .bind(&salt)
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to update user credentials");

        // リクエストを作成
        let request = ChangePasswordRequest {
            current_password: current_password.to_string(),
            new_password: "newpassword456".to_string(),
        };

        // ハンドラを呼び出し
        let result = change_password(State(pool.clone()), Extension(user), Json(request)).await;

        // 結果を検証
        assert!(result.is_ok(), "Password change should succeed");

        // 新しいパスワードで認証できることを確認
        let updated_credentials = sqlx::query_as::<_, UserCredentials>(
            "SELECT * FROM user_credentials WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to get updated credentials");

        let verification =
            verify_password("newpassword456", &updated_credentials.password_hash).unwrap();
        assert!(verification, "New password verification should succeed");
    }

    #[sqlx::test]
    async fn test_change_password_wrong_current_password(pool: PgPool) {
        // テストユーザーを作成
        let user_id = seed_test_user(&pool, "change_password_wrong").await;

        // テスト用のユーザーを取得
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get test user");

        // 既存のパスワードをセットアップ
        let current_password = "password123";
        let (password_hash, salt) = hash_password(current_password).unwrap();

        // ユーザー認証情報を更新
        sqlx::query("UPDATE user_credentials SET password_hash = $1, salt = $2 WHERE user_id = $3")
            .bind(&password_hash)
            .bind(&salt)
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to update user credentials");

        // 間違ったパスワードでリクエストを作成
        let request = ChangePasswordRequest {
            current_password: "wrongpassword".to_string(),
            new_password: "newpassword456".to_string(),
        };

        // ハンドラを呼び出し
        let result = change_password(State(pool.clone()), Extension(user), Json(request)).await;

        // 結果を検証 - エラーが発生するはず
        assert!(
            result.is_err(),
            "Password change should fail with wrong current password"
        );
        if let Err(err) = result {
            assert!(
                matches!(err, AppError::Forbidden),
                "Error should be Forbidden"
            );
        }
    }

    #[sqlx::test]
    async fn test_change_password_invalid_new_password(pool: PgPool) {
        // テストユーザーを作成
        let user_id = seed_test_user(&pool, "change_password_invalid").await;

        // テスト用のユーザーを取得
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get test user");

        // 既存のパスワードをセットアップ
        let current_password = "password123";
        let (password_hash, salt) = hash_password(current_password).unwrap();

        // ユーザー認証情報を更新
        sqlx::query("UPDATE user_credentials SET password_hash = $1, salt = $2 WHERE user_id = $3")
            .bind(&password_hash)
            .bind(&salt)
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to update user credentials");

        // 短すぎる新しいパスワードでリクエストを作成
        let request = ChangePasswordRequest {
            current_password: current_password.to_string(),
            new_password: "short".to_string(), // 8文字未満
        };

        // ハンドラを呼び出し
        let result = change_password(State(pool.clone()), Extension(user), Json(request)).await;

        // 結果を検証 - バリデーションエラーが発生するはず
        assert!(
            result.is_err(),
            "Password change should fail with invalid new password"
        );
        if let Err(err) = result {
            assert!(
                matches!(err, AppError::Validation(_)),
                "Error should be ValidationError"
            );
        }
    }
}
