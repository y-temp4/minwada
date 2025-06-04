use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// Request DTOs

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateCommentRequest {
    #[validate(length(
        min = 1,
        max = 5000,
        message = "Content must be between 1 and 5000 characters"
    ))]
    pub content: String,

    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateCommentRequest {
    #[validate(length(
        min = 1,
        max = 5000,
        message = "Content must be between 1 and 5000 characters"
    ))]
    pub content: String,
}

// Response DTOs

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct CommentResponse {
    pub id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user: CommentUser,
    pub parent_id: Option<Uuid>,
    pub replies: Vec<CommentResponse>,
    pub reply_count: u64,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct CommentUser {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CommentListResponse {
    pub comments: Vec<CommentResponse>,
    pub total_count: u64,
}

// Database query result structs

#[derive(Debug, sqlx::FromRow)]
pub struct CommentWithUser {
    // Comment fields
    pub id: Uuid,
    pub thread_id: Uuid,
    pub content: String,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // User fields
    pub user_id: Uuid,
    pub username: String,
    pub user_display_name: Option<String>,
    pub user_avatar_url: Option<String>,
}

impl CommentWithUser {
    pub fn to_response(self) -> CommentResponse {
        CommentResponse {
            id: self.id,
            content: self.content,
            created_at: self.created_at,
            updated_at: self.updated_at,
            user: CommentUser {
                id: self.user_id,
                username: self.username,
                display_name: self.user_display_name,
                avatar_url: self.user_avatar_url,
            },
            parent_id: self.parent_id,
            replies: Vec::new(), // Will be populated by the service
            reply_count: 0,      // Will be populated by the service
        }
    }
}
