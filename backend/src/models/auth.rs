use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::validations::username;

// Request DTOs

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(custom(function = "username::username_validator"))]
    pub username: String,

    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    #[validate(length(max = 100, message = "Display name must be less than 100 characters"))]
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,

    #[validate(length(min = 8, max = 100, message = "New password must be between 8 and 100 characters"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RequestPasswordResetRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 8, max = 100, message = "New password must be between 8 and 100 characters"))]
    pub new_password: String,
}

// Response DTOs

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LogoutResponse {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

// JWT Claims

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      // Subject (user ID)
    pub username: String, // Username
    pub email: String,    // Email
    pub exp: usize,       // Expiration time
    pub iat: usize,       // Issued at
    pub iss: String,      // Issuer
}

// OAuth DTOs

#[derive(Debug, Deserialize)]
pub struct GoogleOAuthResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
    pub verified_email: bool,
}

#[derive(Debug, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: String,
}
