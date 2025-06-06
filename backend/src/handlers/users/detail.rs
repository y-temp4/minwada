use axum::{
    extract::{Path, State},
    response::Json,
};
use sqlx::PgPool;

use crate::{
    error::AppError,
    models::{common::ErrorResponse, users::PublicUserResponse, User},
};

#[utoipa::path(
    get,
    path = "/api/users/{username}",
    params(
        ("username" = String, Path, description = "Username to lookup")
    ),
    responses(
        (status = 200, description = "User profile found", body = PublicUserResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    tag = "users"
)]
pub async fn get_user_by_username(
    State(pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<PublicUserResponse>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(PublicUserResponse::from(user)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_thread, seed_test_user};
    use axum::{
        extract::{Path, State},
        response::Json,
    };
    use sqlx::PgPool;
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_get_user_by_username_success(pool: PgPool) -> Result<(), AppError> {
        // 存在するユーザー名でプロフィールが正常に取得できることをテスト
        let user_id = seed_test_user(&pool, "detail_test").await;

        // ユーザー名を取得
        let username = sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await?;

        // APIエンドポイントを呼び出す
        let response = get_user_by_username(State(pool), Path(username.clone())).await?;

        let user_response = response.0;

        // 基本的なプロパティが正しく返されていることを確認
        assert_eq!(user_response.id, user_id);
        assert_eq!(user_response.username, username);

        // 機密情報が含まれていないことを確認
        // PublicUserResponseにはemailフィールドが存在しないことを確認
        let response_json = serde_json::to_string(&user_response)?;
        assert!(
            !response_json.contains("email"),
            "Response should not contain email"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_user_by_username_not_found(pool: PgPool) -> Result<(), anyhow::Error> {
        // 存在しないユーザー名の場合、NotFoundエラーが返されることをテスト
        let non_existent_username = format!("non_existent_user_{}", Uuid::new_v4());

        // APIエンドポイントを呼び出す
        let result = get_user_by_username(State(pool), Path(non_existent_username)).await;

        // NotFoundエラーが返されることを確認
        assert!(matches!(result, Err(AppError::NotFound)));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_user_by_username_response_format(pool: PgPool) -> Result<(), AppError> {
        // レスポンスの形式が期待通りであることをテスト
        let user_id = seed_test_user(&pool, "response_format_test").await;

        // テストユーザーにスレッドを追加してデータを豊富にする
        // スレッドの作成自体はテストには直接関係ないが、より実際の利用状況に近い状態を作る
        create_test_thread(&pool, user_id, "Test Thread", "Test Content").await;

        // ユーザー名を取得
        let username = sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await?;

        // APIエンドポイントを呼び出す
        let response = get_user_by_username(State(pool), Path(username)).await?;

        let user_response = response.0;

        // PublicUserResponseの形式が正しいことを確認
        assert!(
            user_response.id.to_string().len() > 0,
            "ID should be present"
        );
        assert!(
            !user_response.username.is_empty(),
            "Username should be present"
        );
        assert!(
            user_response.created_at.to_string().len() > 0,
            "Created at should be present"
        );

        // 機密情報が含まれていないことを確認
        let response_json = serde_json::to_string(&user_response)?;
        assert!(
            !response_json.contains("password"),
            "Response should not contain password"
        );
        assert!(
            !response_json.contains("password_hash"),
            "Response should not contain password_hash"
        );
        assert!(
            !response_json.contains("salt"),
            "Response should not contain salt"
        );
        assert!(
            !response_json.contains("token"),
            "Response should not contain token"
        );
        assert!(
            !response_json.contains("email"),
            "Response should not contain email"
        );

        Ok(())
    }
}
