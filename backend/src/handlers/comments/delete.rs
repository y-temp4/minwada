use axum::{extract::Extension, extract::Path, extract::State, http::StatusCode};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, models::common::ErrorResponse, models::User};

#[utoipa::path(
    delete,
    path = "/api/comments/{id}",
    params(
        ("id" = Uuid, Path, description = "Comment ID")
    ),
    responses(
        (status = 204, description = "Comment deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Comment not found", body = ErrorResponse)
    ),
    tag = "comments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_comment(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Extension(current_user): Extension<User>,
) -> Result<StatusCode, AppError> {
    // Check if comment exists and user owns it, then delete
    let deleted_rows = sqlx::query("DELETE FROM comments WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(current_user.id)
        .execute(&pool)
        .await?;

    if deleted_rows.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}