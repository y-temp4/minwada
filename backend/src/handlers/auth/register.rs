use axum::{extract::State, http::StatusCode, Json};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    auth::{jwt::create_jwt_token, password::hash_password},
    config::Config,
    error::AppError,
    models::{
        auth::{AuthResponse, RegisterRequest, UserInfo},
        common::ErrorResponse,
        User,
    },
    utils::{self, email_sender, token_hash::hash_refresh_token},
};

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 409, description = "User already exists", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    // Validate input
    payload.validate()?;

    // Check if user already exists
    let existing_user =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 OR username = $2")
            .bind(&payload.email)
            .bind(&payload.username)
            .fetch_optional(&pool)
            .await?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("User already exists".to_string()));
    }

    // Hash password
    let (password_hash, salt) = hash_password(&payload.password)?;

    // Start transaction
    let mut tx = pool.begin().await?;

    // Create user
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, display_name, email_verified)
        VALUES ($1, $2, $3, false)
        RETURNING *
        "#,
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&payload.display_name)
    .fetch_one(&mut *tx)
    .await?;

    // Create user credentials
    sqlx::query(
        r#"
        INSERT INTO user_credentials (user_id, password_hash, salt)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user.id)
    .bind(&password_hash)
    .bind(&salt)
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    // Generate verification token and send verification email
    let mut tx = pool.begin().await?;
    let verification_token = email_sender::start_verification_flow(&user, &mut tx).await?;
    tx.commit().await?;

    // ユーザー情報をクローンして非同期処理に渡す
    let user_clone = user.clone();
    // 非同期でメール送信
    tokio::spawn(async move {
        let _ = email_sender::send_verification_email(&user_clone, &verification_token).await;
    });

    // Generate tokens
    let config = Config::from_env()?;
    let access_token = create_jwt_token(
        &user.id.to_string(),
        &user.username,
        &user.email,
        &config.jwt_secret,
        15, // 15 minutes
    )?;

    let refresh_token = utils::generate_secure_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    // Store refresh token
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, NOW() + INTERVAL '7 days')
        "#,
    )
    .bind(user.id)
    .bind(&refresh_token_hash)
    .execute(&pool)
    .await?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 900, // 15 minutes in seconds
        user: UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            email_verified: user.email_verified,
            created_at: user.created_at,
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::User, test_utils::seed_test_user};
    use axum::http::StatusCode;

    #[sqlx::test]
    async fn test_register_success(pool: PgPool) {
        // テスト用の登録リクエストを作成
        let register_request = RegisterRequest {
            username: "newuser123".to_string(),
            email: "newuser@example.com".to_string(),
            password: "password123".to_string(),
            display_name: Some("New Test User".to_string()),
        };

        // ハンドラを直接呼び出し
        let result = register(State(pool.clone()), Json(register_request)).await;

        // レスポンスを検証
        assert!(result.is_ok(), "register should return Ok");
        let response = result.unwrap();

        // ステータスコードを確認
        assert_eq!(response.0, StatusCode::CREATED);

        // レスポンスボディを確認
        let auth_response = response.1 .0;
        assert!(!auth_response.access_token.is_empty());
        assert!(!auth_response.refresh_token.is_empty());
        assert_eq!(auth_response.token_type, "Bearer");
        assert_eq!(auth_response.expires_in, 900);
        assert_eq!(auth_response.user.username, "newuser123");
        assert_eq!(auth_response.user.email, "newuser@example.com");
        assert_eq!(
            auth_response.user.display_name,
            Some("New Test User".to_string())
        );

        // データベースにユーザーが作成されたことを確認
        let created_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind("newuser123")
            .fetch_one(&pool)
            .await
            .expect("Failed to get created user");

        // ユーザー認証情報が作成されたことを確認
        let credentials_exist = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM user_credentials WHERE user_id = $1)",
        )
        .bind(created_user.id)
        .fetch_one(&pool)
        .await
        .expect("Failed to check credentials");

        assert!(credentials_exist, "User credentials should exist");

        // リフレッシュトークンが作成されたことを確認
        let token_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM refresh_tokens WHERE user_id = $1)",
        )
        .bind(created_user.id)
        .fetch_one(&pool)
        .await
        .expect("Failed to check refresh token");

        assert!(token_exists, "Refresh token should exist");
    }

    #[sqlx::test]
    async fn test_register_existing_username(pool: PgPool) {
        // 既存のテストユーザーを作成
        let user_id = seed_test_user(&pool, "existing_user").await;

        // 既存のユーザー名で登録リクエストを作成
        let register_request = RegisterRequest {
            username: "testuser_existing_user".to_string(), // 既存のユーザー名
            email: "new_email@example.com".to_string(),     // 新しいメールアドレス
            password: "password123".to_string(),
            display_name: None,
        };

        // ハンドラを直接呼び出し
        let result = register(State(pool.clone()), Json(register_request)).await;

        // エラーが返されることを確認
        assert!(result.is_err(), "Should return error for existing username");

        // 競合エラーを確認
        match result {
            Err(AppError::Conflict(_)) => {} // 期待通り
            _ => panic!("Expected Conflict error"),
        }
    }

    #[sqlx::test]
    async fn test_register_existing_email(pool: PgPool) {
        // 既存のテストユーザーを作成
        seed_test_user(&pool, "email_test").await;

        // 既存のメールアドレスで登録リクエストを作成
        let register_request = RegisterRequest {
            username: "new_username".to_string(), // 新しいユーザー名
            email: "test_email_test@example.com".to_string(), // 既存のメールアドレス
            password: "password123".to_string(),
            display_name: None,
        };

        // ハンドラを直接呼び出し
        let result = register(State(pool.clone()), Json(register_request)).await;

        // エラーが返されることを確認
        assert!(result.is_err(), "Should return error for existing email");

        // 競合エラーを確認
        match result {
            Err(AppError::Conflict(_)) => {} // 期待通り
            _ => panic!("Expected Conflict error"),
        }
    }

    #[sqlx::test]
    async fn test_register_invalid_password(pool: PgPool) {
        // 短すぎるパスワードで登録リクエストを作成
        let register_request = RegisterRequest {
            username: "valid_user".to_string(),
            email: "valid@example.com".to_string(),
            password: "short".to_string(), // 8文字未満のパスワード
            display_name: None,
        };

        // ハンドラを直接呼び出し
        let result = register(State(pool.clone()), Json(register_request)).await;

        // エラーが返されることを確認
        assert!(result.is_err(), "Should return error for invalid password");

        // バリデーションエラーを確認
        match result {
            Err(AppError::Validation(_)) => {} // 期待通り
            _ => panic!("Expected Validation error"),
        }
    }

    #[sqlx::test]
    async fn test_register_invalid_email(pool: PgPool) {
        // 無効なメールアドレスで登録リクエストを作成
        let register_request = RegisterRequest {
            username: "valid_user".to_string(),
            email: "invalid-email".to_string(), // 無効なメールアドレス
            password: "password123".to_string(),
            display_name: None,
        };

        // ハンドラを直接呼び出し
        let result = register(State(pool.clone()), Json(register_request)).await;

        // エラーが返されることを確認
        assert!(result.is_err(), "Should return error for invalid email");

        // バリデーションエラーを確認
        match result {
            Err(AppError::Validation(_)) => {} // 期待通り
            _ => panic!("Expected Validation error"),
        }
    }

    #[sqlx::test]
    async fn test_register_invalid_username_length(pool: PgPool) {
        // 短すぎるユーザー名で登録リクエストを作成
        let register_request = RegisterRequest {
            username: "ab".to_string(), // 短すぎるユーザー名
            email: "valid@example.com".to_string(),
            password: "password123".to_string(),
            display_name: None,
        };

        // ハンドラを直接呼び出し
        let result = register(State(pool.clone()), Json(register_request)).await;

        // エラーが返されることを確認
        assert!(
            result.is_err(),
            "Should return error for invalid username length"
        );

        // バリデーションエラーを確認
        match result {
            Err(AppError::Validation(_)) => {} // 期待通り
            _ => panic!("Expected Validation error"),
        }
    }

    #[sqlx::test]
    async fn test_register_reserved_username(pool: PgPool) {
        // 予約語を含むユーザー名で登録リクエストを作成
        let register_request = RegisterRequest {
            username: "admin".to_string(), // 予約語のユーザー名
            email: "valid@example.com".to_string(),
            password: "password123".to_string(),
            display_name: None,
        };

        // ハンドラを直接呼び出し
        let result = register(State(pool.clone()), Json(register_request)).await;

        // エラーが返されることを確認
        assert!(result.is_err(), "Should return error for reserved username");

        // バリデーションエラーを確認
        match result {
            Err(AppError::Validation(_)) => {} // 期待通り
            _ => panic!("Expected Validation error"),
        }
    }

    #[sqlx::test]
    async fn test_register_invalid_username_characters(pool: PgPool) {
        // 無効な文字を含むユーザー名で登録リクエストを作成
        let register_request = RegisterRequest {
            username: "user@name".to_string(), // 特殊文字を含むユーザー名
            email: "valid@example.com".to_string(),
            password: "password123".to_string(),
            display_name: None,
        };

        // ハンドラを直接呼び出し
        let result = register(State(pool.clone()), Json(register_request)).await;

        // エラーが返されることを確認
        assert!(
            result.is_err(),
            "Should return error for invalid username characters"
        );

        // バリデーションエラーを確認
        match result {
            Err(AppError::Validation(_)) => {} // 期待通り
            _ => panic!("Expected Validation error"),
        }
    }
}
