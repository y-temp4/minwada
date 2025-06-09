use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Argon2 error: {0}")]
    Argon2(String),

    // #[error("OAuth error: {0}")]
    // OAuth(String),
    //
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Authentication error: {0}")]
    Unauthorized(String),

    #[error("Forbidden")]
    Forbidden,

    #[error("Resource not found")]
    NotFound,

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("UUID parse error: {0}")]
    UuidParse(#[from] uuid::Error),

    #[error("Email verification required")]
    EmailVerificationRequired,
}

// Manual implementation of From trait for argon2 errors
impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        AppError::Argon2(err.to_string())
    }
}

// Manual implementation for config errors
impl From<Box<dyn std::error::Error>> for AppError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AppError::Config(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error occurred".to_string(),
                )
            }
            AppError::Jwt(_) => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            AppError::Argon2(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Password hashing error".to_string(),
            ),
            // AppError::OAuth(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Validation(ref errors) => {
                let validation_errors: Vec<String> = errors
                    .field_errors()
                    .iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |error| {
                            format!(
                                "{}: {}",
                                field,
                                error.message.as_ref().unwrap_or(&"Invalid value".into())
                            )
                        })
                    })
                    .collect();
                (
                    StatusCode::BAD_REQUEST,
                    format!("Validation errors: {}", validation_errors.join(", ")),
                )
            }
            AppError::Unauthorized(ref msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            AppError::Conflict(ref msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::NotImplemented(ref msg) => (StatusCode::NOT_IMPLEMENTED, msg.clone()),
            AppError::Config(ref msg) => {
                tracing::error!("Configuration error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Configuration error".to_string(),
                )
            }
            AppError::EmailVerificationRequired => (
                StatusCode::FORBIDDEN,
                "メールアドレスの認証が必要です".to_string(),
            ),
            AppError::Reqwest(ref err) => {
                tracing::error!("HTTP client error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "External service error".to_string(),
                )
            }
            AppError::SerdeJson(ref err) => {
                tracing::error!("JSON serialization error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "JSON processing error".to_string(),
                )
            }
            AppError::UuidParse(ref err) => {
                tracing::error!("UUID parse error: {:?}", err);
                (StatusCode::BAD_REQUEST, "Invalid UUID format".to_string())
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
