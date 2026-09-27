use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use common::{ApiResponse, AppError, PaginatedResponse, Pagination};
use serde::Deserialize;
use services::work_service::WorkDto;

#[derive(Debug, Deserialize)]
pub struct ListWorksQuery {
    #[serde(flatten)]
    pagination: Pagination,
    category: Option<String>,
    creator_id: Option<i32>,
    q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkInput {
    image_url: String,
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/works", get(list_works).post(create_work))
}

async fn list_works(
    State(state): State<AppState>,
    Query(query): Query<ListWorksQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<WorkDto>>>, AppError> {
    let page = query.pagination.page();
    let page_size = query.pagination.page_size();
    let (items, total) = state
        .works
        .list_paginated(page, page_size, query.category, query.creator_id, query.q)
        .await?;
    Ok(Json(ApiResponse::success(PaginatedResponse::new(
        items, total, page, page_size,
    ))))
}

async fn create_work(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(input): Json<CreateWorkInput>,
) -> Result<Json<ApiResponse<WorkDto>>, AppError> {
    let work = state
        .works
        .create(
            claims.sub,
            input.image_url,
            input.title,
            input.description,
            input.category,
        )
        .await?;
    Ok(Json(ApiResponse::success(work)))
}
