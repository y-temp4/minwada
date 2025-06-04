use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        threads::{CreateThreadRequest, ThreadResponse, ThreadWithUser},
        User,
    },
};

#[utoipa::path(
    post,
    path = "/api/threads",
    request_body = CreateThreadRequest,
    responses(
        (status = 201, description = "Thread created successfully", body = ThreadResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "threads",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_thread(
    State(pool): State<PgPool>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<CreateThreadRequest>,
) -> Result<(StatusCode, Json<ThreadResponse>), AppError> {
    // Validate input
    payload.validate()?;

    // Create thread
    let thread = sqlx::query_as::<_, ThreadWithUser>(
        r#"
        INSERT INTO threads (user_id, title, content)
        VALUES ($1, $2, $3)
        RETURNING 
            id, title, content, created_at, updated_at,
            $1 as user_id, $4 as username, $5 as user_display_name, $6 as user_avatar_url,
            0::bigint as comment_count
        "#,
    )
    .bind(current_user.id)
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&current_user.username)
    .bind(&current_user.display_name)
    .bind(&current_user.avatar_url)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(ThreadResponse::from(thread))))
}
