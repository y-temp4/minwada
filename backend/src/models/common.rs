use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub status: u16,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SuccessResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_limit")]
    pub limit: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self { page: 1, limit: 20 }
    }
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: u64, page: u32, limit: u32) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

        Self {
            data,
            total,
            page,
            limit,
            total_pages,
        }
    }
}
