use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use common::{ApiResponse, AppError, PaginatedResponse, Pagination};
use serde::Deserialize;
use services::admin_service::{AdminUserDto, PlatformStats};

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<u64>,
    page_size: Option<u64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(list_users))
        .route("/admin/stats", get(stats))
}

async fn list_users(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<AdminUserDto>>>, AppError> {
    require_admin(&claims)?;
    let pagination = Pagination {
        page: query.page,
        page_size: query.page_size,
    };
    let page = pagination.page();
    let page_size = pagination.page_size();
    let (items, total) = state.admin.list_users_paginated(page, page_size).await?;
    Ok(Json(ApiResponse::success(PaginatedResponse::new(
        items, total, page, page_size,
    ))))
}

async fn stats(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<PlatformStats>>, AppError> {
    require_admin(&claims)?;
    let stats = state.admin.stats().await?;
    Ok(Json(ApiResponse::success(stats)))
}

fn require_admin(claims: &services::auth_service::Claims) -> Result<(), AppError> {
    if claims.role == "admin" {
        Ok(())
    } else {
        Err(AppError::Forbidden("admin access required".into()))
    }
}
