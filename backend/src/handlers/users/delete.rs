use axum::{
    extract::{Extension, State},
    http::StatusCode,
};
use sqlx::PgPool;
use tracing::error;

use crate::{
    error::AppError,
    models::{common::ErrorResponse, User},
};

/// Delete the current user account
///
/// Delete the authenticated user's account and all associated data.
#[utoipa::path(
    delete,
    path = "/api/users/me",
    tag = "users",
    responses(
        (status = 200, description = "User successfully deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn delete_user(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
) -> Result<StatusCode, AppError> {
    // Start a transaction to ensure all operations succeed or fail together
    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Internal(e.to_string())
    })?;

    // Delete user credentials
    sqlx::query!(
        r#"
        DELETE FROM user_credentials
        WHERE user_id = $1
        "#,
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to delete user credentials: {}", e);
        AppError::Internal(e.to_string())
    })?;

    // Delete OAuth accounts
    sqlx::query!(
        r#"
        DELETE FROM oauth_accounts
        WHERE user_id = $1
        "#,
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to delete OAuth accounts: {}", e);
        AppError::Internal(e.to_string())
    })?;

    // Delete refresh tokens
    sqlx::query!(
        r#"
        DELETE FROM refresh_tokens
        WHERE user_id = $1
        "#,
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to delete refresh tokens: {}", e);
        AppError::Internal(e.to_string())
    })?;

    // Delete comments
    sqlx::query!(
        r#"
        DELETE FROM comments
        WHERE user_id = $1
        "#,
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to delete comments: {}", e);
        AppError::Internal(e.to_string())
    })?;

    // Delete threads
    sqlx::query!(
        r#"
        DELETE FROM threads
        WHERE user_id = $1
        "#,
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to delete threads: {}", e);
        AppError::Internal(e.to_string())
    })?;

    // Delete user
    sqlx::query!(
        r#"
        DELETE FROM users
        WHERE id = $1
        "#,
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to delete user: {}", e);
        AppError::Internal(e.to_string())
    })?;

    // Commit the transaction
    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        AppError::Internal(e.to_string())
    })?;

    // Return 200 OK status
    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_delete_user_success(pool: PgPool) {
        // Create a test user directly in the database for testing
        let user_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, email_verified, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            "#,
            user_id,
            "testdelete",
            "testdelete@example.com",
            false
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create a mock user for the handler
        let user = User {
            id: user_id,
            username: "testdelete".to_string(),
            email: "testdelete@example.com".to_string(),
            display_name: None,
            avatar_url: None,
            email_verified: false,
            email_verified_at: None,
            verification_token: None,
            verification_token_expires_at: None,
            password_reset_token: None,
            password_reset_token_expires_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Delete the user
        let response = delete_user(State(pool.clone()), Extension(user)).await;
        assert!(response.is_ok());

        // Verify user no longer exists
        let user_exists = sqlx::query!(
            r#"
            SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as "exists!"
            "#,
            user_id
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .exists;

        assert!(!user_exists);
    }
}
