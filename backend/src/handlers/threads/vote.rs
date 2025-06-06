use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{common::ErrorResponse, User},
};

#[derive(Deserialize, ToSchema)]
pub struct VoteRequest {
    pub vote_type: String, // "upvote" or "downvote"
}

#[utoipa::path(
    post,
    path = "/api/threads/{id}/vote",
    request_body = VoteRequest,
    responses(
        (status = 200, description = "Voted successfully", body = ErrorResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Thread not found", body = ErrorResponse)
    ),
    tag = "threads",
    security(("bearer_auth" = []))
)]
pub async fn vote_thread(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<VoteRequest>,
) -> Result<StatusCode, AppError> {
    // スレッド存在確認
    let exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM threads WHERE id = $1)")
            .bind(id)
            .fetch_optional(&pool)
            .await?;

    if !exists.unwrap_or(false) {
        return Err(AppError::NotFound);
    }
    // upvote/downvote以外はエラー
    if payload.vote_type != "upvote" && payload.vote_type != "downvote" {
        return Err(AppError::BadRequest("Invalid vote_type".to_string()));
    }

    // 既存投票取得
    let existing = sqlx::query_scalar::<_, String>(
        "SELECT vote_type FROM votes WHERE user_id = $1 AND thread_id = $2",
    )
    .bind(current_user.id)
    .bind(id)
    .fetch_optional(&pool)
    .await?;

    if let Some(current) = existing {
        if current == payload.vote_type {
            // 同じ投票なら削除（トグル）
            sqlx::query("DELETE FROM votes WHERE user_id = $1 AND thread_id = $2")
                .bind(current_user.id)
                .bind(id)
                .execute(&pool)
                .await?;
            return Ok(StatusCode::NO_CONTENT);
        } else {
            // 種類が違う場合は更新
            sqlx::query(
                "UPDATE votes SET vote_type = $1, updated_at = NOW() WHERE user_id = $2 AND thread_id = $3"
            )
            .bind(&payload.vote_type)
            .bind(current_user.id)
            .bind(id)
            .execute(&pool)
            .await?;
            return Ok(StatusCode::OK);
        }
    } else {
        // 新規投票
        sqlx::query("INSERT INTO votes (user_id, thread_id, vote_type) VALUES ($1, $2, $3)")
            .bind(current_user.id)
            .bind(id)
            .bind(&payload.vote_type)
            .execute(&pool)
            .await?;
        return Ok(StatusCode::OK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::User;
    use axum::extract::State;
    use axum::Extension;
    use axum::Json;
    use sqlx::PgPool;
    use uuid::Uuid;

    // テスト用ユーザーとスレッド作成のユーティリティ
    async fn setup_user_and_thread(pool: &PgPool) -> (User, Uuid) {
        let user_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, username, email, email_verified) VALUES ($1, 'voteuser', 'vote@example.com', true)",
            user_id
        ).execute(pool).await.unwrap();
        sqlx::query!(
            "INSERT INTO threads (id, user_id, title, upvote_count, downvote_count) VALUES ($1, $2, 'vote thread', 0, 0)",
            thread_id,
            user_id
        )
        .execute(pool)
        .await
        .unwrap();
        let user = User {
            id: user_id,
            username: "voteuser".to_string(),
            email: "vote@example.com".to_string(),
            display_name: None,
            avatar_url: None,
            email_verified: true,
            email_verified_at: Some(chrono::Utc::now()),
            verification_token: None,
            verification_token_expires_at: None,
            password_reset_token: None,
            password_reset_token_expires_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        (user, thread_id)
    }

    #[sqlx::test]
    async fn test_投票が正常にできること(pool: PgPool) {
        // 正常なupvote
        let (user, thread_id) = setup_user_and_thread(&pool).await;
        let req = VoteRequest {
            vote_type: "upvote".to_string(),
        };
        let result = vote_thread(
            State(pool.clone()),
            Path(thread_id),
            Extension(user.clone()),
            Json(req),
        )
        .await;
        assert!(result.is_ok());
        // DBに投票が記録されていること
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM votes WHERE user_id = $1 AND thread_id = $2 AND vote_type = 'upvote'")
            .bind(user.id).bind(thread_id).fetch_one(&pool).await.unwrap();
        assert_eq!(count, 1);
    }

    #[sqlx::test]
    async fn test_同じ投票タイプでトグル削除できること(pool: PgPool) {
        // upvote→upvoteで削除
        let (user, thread_id) = setup_user_and_thread(&pool).await;
        // 1回目upvote
        let req = VoteRequest {
            vote_type: "upvote".to_string(),
        };
        vote_thread(
            State(pool.clone()),
            Path(thread_id),
            Extension(user.clone()),
            Json(req),
        )
        .await
        .unwrap();
        // 2回目upvote（削除）
        let req2 = VoteRequest {
            vote_type: "upvote".to_string(),
        };
        let res = vote_thread(
            State(pool.clone()),
            Path(thread_id),
            Extension(user.clone()),
            Json(req2),
        )
        .await;
        assert!(res.is_ok());
        // DBに投票がないこと
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM votes WHERE user_id = $1 AND thread_id = $2")
                .bind(user.id)
                .bind(thread_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 0);
    }

    #[sqlx::test]
    async fn test_投票タイプ変更で更新されること(pool: PgPool) {
        // upvote→downvote
        let (user, thread_id) = setup_user_and_thread(&pool).await;
        let req = VoteRequest {
            vote_type: "upvote".to_string(),
        };
        vote_thread(
            State(pool.clone()),
            Path(thread_id),
            Extension(user.clone()),
            Json(req),
        )
        .await
        .unwrap();
        let req2 = VoteRequest {
            vote_type: "downvote".to_string(),
        };
        let res = vote_thread(
            State(pool.clone()),
            Path(thread_id),
            Extension(user.clone()),
            Json(req2),
        )
        .await;
        assert!(res.is_ok());
        let vt: Option<String> =
            sqlx::query_scalar("SELECT vote_type FROM votes WHERE user_id = $1 AND thread_id = $2")
                .bind(user.id)
                .bind(thread_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(vt.unwrap(), "downvote");
    }

    #[sqlx::test]
    async fn test_不正な投票タイプはエラーになること(pool: PgPool) {
        // vote_typeが不正
        let (user, thread_id) = setup_user_and_thread(&pool).await;
        let req = VoteRequest {
            vote_type: "invalid".to_string(),
        };
        let res = vote_thread(
            State(pool.clone()),
            Path(thread_id),
            Extension(user),
            Json(req),
        )
        .await;
        assert!(res.is_err());
    }

    #[sqlx::test]
    async fn test_存在しないスレッドはエラーになること(pool: PgPool) {
        // 存在しないスレッドID
        let user_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, username, email, email_verified) VALUES ($1, 'nouser', 'nouser@example.com', true)",
            user_id
        ).execute(&pool).await.unwrap();
        let user = User {
            id: user_id,
            username: "nouser".to_string(),
            email: "nouser@example.com".to_string(),
            display_name: None,
            avatar_url: None,
            email_verified: true,
            email_verified_at: Some(chrono::Utc::now()),
            verification_token: None,
            verification_token_expires_at: None,
            password_reset_token: None,
            password_reset_token_expires_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let req = VoteRequest {
            vote_type: "upvote".to_string(),
        };
        let res = vote_thread(
            State(pool.clone()),
            Path(Uuid::new_v4()),
            Extension(user),
            Json(req),
        )
        .await;
        assert!(res.is_err());
    }
}
