use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    error::AppError,
    models::{common::ErrorResponse, User},
    utils::email_sender,
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateEmailRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateEmailResponse {
    message: String,
}

#[utoipa::path(
    put,
    path = "/api/users/me/email",
    request_body = UpdateEmailRequest,
    responses(
        (status = 200, description = "Email update verification sent", body = UpdateEmailResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Email already exists", body = ErrorResponse)
    ),
    tag = "users",
    security(
        ("jwt_auth" = [])
    )
)]
pub async fn update_email(
    State(pool): State<PgPool>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<UpdateEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    payload.validate()?;

    // Check if email is different
    if current_user.email == payload.email {
        return Err(AppError::BadRequest(
            "New email must be different from current email".to_string(),
        ));
    }

    // Check if email is taken by another user
    let existing_user =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1 AND id != $2")
            .bind(&payload.email)
            .bind(current_user.id)
            .fetch_one(&pool)
            .await?;

    if existing_user > 0 {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    // Start transaction to update email and create verification token
    let mut tx = pool.begin().await?;

    // Update user email and reset verification status
    let updated_user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users 
        SET 
            email = $2,
            email_verified = false,
            email_verified_at = NULL,
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(current_user.id)
    .bind(&payload.email)
    .fetch_one(&mut *tx)
    .await?;

    // Generate verification token and prepare for email sending
    let verification_token = email_sender::start_verification_flow(&updated_user, &mut tx).await?;

    // Commit transaction
    tx.commit().await?;

    // Send verification email asynchronously
    tokio::spawn(async move {
        let _ = email_sender::send_verification_email(&updated_user, &verification_token).await;
    });

    Ok((
        StatusCode::OK,
        Json(UpdateEmailResponse {
            message: "Email updated. Verification email has been sent to your new email address."
                .to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;

    use crate::test_utils::seed_test_user;

    #[sqlx::test]
    async fn test_update_email_success(pool: PgPool) {
        // テストユーザーを作成
        let user_id = seed_test_user(&pool, "email_update_test").await;

        // テスト用のユーザーを取得
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        // メールアドレス更新リクエスト
        let request = UpdateEmailRequest {
            email: "test_new@example.com".to_string(),
        };

        // APIを実行
        let result = update_email(State(pool.clone()), Extension(user), Json(request)).await;

        // 結果を確認
        if let Err(ref e) = result {
            eprintln!("エラー内容: {:?}", e);
        }
        assert!(result.is_ok());

        // データベースの状態を確認
        let updated_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(updated_user.email, "test_new@example.com".to_string());
        assert!(!updated_user.email_verified);
        assert!(updated_user.email_verified_at.is_none());
        assert!(updated_user.verification_token.is_some());
        assert!(updated_user.verification_token_expires_at.is_some());
    }

    #[sqlx::test]
    async fn test_update_email_same_email(pool: PgPool) {
        // テストユーザーを作成
        let user_id = seed_test_user(&pool, "email_update_same_test").await;

        // テスト用のユーザーを取得
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        // 同じメールアドレスでリクエスト
        let request = UpdateEmailRequest {
            email: user.email.clone(),
        };

        // APIを実行
        let result = update_email(State(pool.clone()), Extension(user), Json(request)).await;

        // エラーが返されることを確認
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, AppError::BadRequest(_)));
        }
    }
}
