use axum::{
    extract::{Path, Query, State, Extension},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        threads::{CreateThreadRequest, UpdateThreadRequest, ThreadResponse, ThreadListResponse, ThreadWithUser},
        common::PaginatedResponse,
        User,
    },
};

#[derive(serde::Deserialize)]
pub struct ThreadQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/api/threads",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<u32>, Query, description = "Number of items per page (default: 20)")
    ),
    responses(
        (status = 200, description = "List of threads", body = ThreadListResponse)
    ),
    tag = "threads"
)]
pub async fn get_threads(
    State(pool): State<PgPool>,
    Query(query): Query<ThreadQuery>,
) -> Result<Json<ThreadListResponse>, AppError> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    // Get total count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM threads")
        .fetch_one(&pool)
        .await?;

    // Get threads with user information and comment count
    let threads = sqlx::query_as::<_, ThreadWithUser>(
        r#"
        SELECT 
            t.id, t.title, t.content, t.created_at, t.updated_at,
            u.id as user_id, u.username, u.display_name as user_display_name, u.avatar_url as user_avatar_url,
            COUNT(c.id)::bigint as comment_count
        FROM threads t
        JOIN users u ON t.user_id = u.id
        LEFT JOIN comments c ON t.id = c.thread_id
        GROUP BY t.id, u.id, u.username, u.display_name, u.avatar_url
        ORDER BY t.created_at DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&pool)
    .await?;

    let thread_responses: Vec<ThreadResponse> = threads.into_iter().map(ThreadResponse::from).collect();

    let paginated_response = PaginatedResponse::new(
        thread_responses,
        total as u64,
        page,
        limit,
    );

    Ok(Json(ThreadListResponse {
        threads: paginated_response,
    }))
}

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
        "#
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

#[utoipa::path(
    get,
    path = "/api/threads/{id}",
    params(
        ("id" = Uuid, Path, description = "Thread ID")
    ),
    responses(
        (status = 200, description = "Thread details", body = ThreadResponse),
        (status = 404, description = "Thread not found", body = ErrorResponse)
    ),
    tag = "threads"
)]
pub async fn get_thread(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ThreadResponse>, AppError> {
    let thread = sqlx::query_as::<_, ThreadWithUser>(
        r#"
        SELECT 
            t.id, t.title, t.content, t.created_at, t.updated_at,
            u.id as user_id, u.username, u.display_name as user_display_name, u.avatar_url as user_avatar_url,
            COUNT(c.id)::bigint as comment_count
        FROM threads t
        JOIN users u ON t.user_id = u.id
        LEFT JOIN comments c ON t.id = c.thread_id
        WHERE t.id = $1
        GROUP BY t.id, u.id, u.username, u.display_name, u.avatar_url
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(ThreadResponse::from(thread)))
}

#[utoipa::path(
    put,
    path = "/api/threads/{id}",
    params(
        ("id" = Uuid, Path, description = "Thread ID")
    ),
    request_body = UpdateThreadRequest,
    responses(
        (status = 200, description = "Thread updated successfully", body = ThreadResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Thread not found", body = ErrorResponse)
    ),
    tag = "threads",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_thread(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<UpdateThreadRequest>,
) -> Result<Json<ThreadResponse>, AppError> {
    // Validate input
    payload.validate()?;

    // Check if thread exists and user owns it
    let existing_thread = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM threads WHERE id = $1 AND user_id = $2)"
    )
    .bind(id)
    .bind(current_user.id)
    .fetch_one(&pool)
    .await?;

    if !existing_thread {
        return Err(AppError::NotFound);
    }

    if payload.title.is_none() && payload.content.is_none() {
        return Err(AppError::BadRequest("No fields to update".to_string()));
    }

    // Update thread with simplified query
    let _updated_thread = sqlx::query(
        r#"
        UPDATE threads 
        SET 
            title = COALESCE($2, title),
            content = COALESCE($3, content),
            updated_at = NOW()
        WHERE id = $1
        "#
    )
    .bind(id)
    .bind(payload.title.as_ref())
    .bind(payload.content.as_ref())
    .execute(&pool)
    .await?;

    // Fetch user information and comment count
    let thread_with_user = sqlx::query_as::<_, ThreadWithUser>(
        r#"
        SELECT 
            t.id, t.title, t.content, t.created_at, t.updated_at,
            u.id as user_id, u.username, u.display_name as user_display_name, u.avatar_url as user_avatar_url,
            COUNT(c.id)::bigint as comment_count
        FROM threads t
        JOIN users u ON t.user_id = u.id
        LEFT JOIN comments c ON t.id = c.thread_id
        WHERE t.id = $1
        GROUP BY t.id, u.id, u.username, u.display_name, u.avatar_url
        "#
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(ThreadResponse::from(thread_with_user)))
}

#[utoipa::path(
    delete,
    path = "/api/threads/{id}",
    params(
        ("id" = Uuid, Path, description = "Thread ID")
    ),
    responses(
        (status = 204, description = "Thread deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Thread not found", body = ErrorResponse)
    ),
    tag = "threads",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_thread(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<User>,
) -> Result<StatusCode, AppError> {
    // Check if thread exists and user owns it, then delete
    let deleted_rows = sqlx::query("DELETE FROM threads WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(current_user.id)
        .execute(&pool)
        .await?;

    if deleted_rows.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
} 
