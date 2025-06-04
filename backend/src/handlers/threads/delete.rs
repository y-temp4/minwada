use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, models::User};

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
