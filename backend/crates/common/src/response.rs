use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

/// 统一 API 响应格式
/// 对齐老系统: { code, message, data }
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: u16,
    pub message: String,
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 200,
            message: "success".into(),
            data,
        }
    }

    pub fn success_with_msg(data: T, message: impl Into<String>) -> Self {
        Self {
            code: 200,
            message: message.into(),
            data,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

/// 分页响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub has_next: bool,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u64, page_size: u64) -> Self {
        let has_next = page * page_size < total;
        Self {
            items,
            total,
            page,
            page_size,
            has_next,
        }
    }
}

/// 查询参数：分页
#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

impl Pagination {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn page_size(&self) -> u64 {
        self.page_size.unwrap_or(20).min(100)
    }
}
