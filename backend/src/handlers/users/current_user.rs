use axum::{extract::Extension, response::Json};

use crate::{
    error::AppError,
    models::{common::ErrorResponse, users::UserResponse, User},
};

#[utoipa::path(
    get,
    path = "/api/users/me",
    responses(
        (status = 200, description = "Current user profile", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_user(
    Extension(current_user): Extension<User>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(UserResponse::from(current_user)))
}
