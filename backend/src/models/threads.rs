use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::common::PaginatedResponse;

// Request DTOs

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateThreadRequest {
    #[validate(length(
        min = 1,
        max = 300,
        message = "Title must be between 1 and 300 characters"
    ))]
    pub title: String,

    #[validate(length(max = 10000, message = "Content must be less than 10000 characters"))]
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateThreadRequest {
    #[validate(length(
        min = 1,
        max = 300,
        message = "Title must be between 1 and 300 characters"
    ))]
    pub title: Option<String>,

    #[validate(length(max = 10000, message = "Content must be less than 10000 characters"))]
    pub content: Option<String>,
}

// Response DTOs

#[derive(Debug, Serialize, ToSchema)]
pub struct ThreadResponse {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user: ThreadUser,
    pub comment_count: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ThreadUser {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ThreadListResponse {
    #[schema(value_type = PaginatedResponse<ThreadResponse>)]
    pub threads: PaginatedResponse<ThreadResponse>,
}

// Database query result structs

#[derive(Debug, sqlx::FromRow)]
pub struct ThreadWithUser {
    // Thread fields
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // User fields
    pub user_id: Uuid,
    pub username: String,
    pub user_display_name: Option<String>,
    pub user_avatar_url: Option<String>,

    // Comment count
    pub comment_count: Option<i64>,
}

impl From<ThreadWithUser> for ThreadResponse {
    fn from(thread: ThreadWithUser) -> Self {
        Self {
            id: thread.id,
            title: thread.title,
            content: thread.content,
            created_at: thread.created_at,
            updated_at: thread.updated_at,
            user: ThreadUser {
                id: thread.user_id,
                username: thread.username,
                display_name: thread.user_display_name,
                avatar_url: thread.user_avatar_url,
            },
            comment_count: thread.comment_count.unwrap_or(0) as u64,
        }
    }
}
