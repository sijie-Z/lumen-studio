use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use common::{ApiResponse, AppError, PaginatedResponse, Pagination};
use rust_decimal::Decimal;
use serde::Deserialize;
use services::review_service::ReviewDto;

#[derive(Debug, Deserialize)]
struct CreateReviewBody {
    appointment_id: i32,
    rating: Decimal,
    content: Option<String>,
    #[serde(default)]
    is_anonymous: bool,
}

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<u64>,
    page_size: Option<u64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/reviews", post(create_review))
        .route("/reviews/creator/{id}", get(list_creator_reviews))
}

async fn create_review(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<CreateReviewBody>,
) -> Result<Json<ApiResponse<ReviewDto>>, AppError> {
    let review = state
        .reviews
        .create(
            claims.sub,
            body.appointment_id,
            body.rating,
            body.content,
            body.is_anonymous,
        )
        .await?;
    Ok(Json(ApiResponse::success(review)))
}

async fn list_creator_reviews(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<ReviewDto>>>, AppError> {
    let pagination = Pagination {
        page: query.page,
        page_size: query.page_size,
    };
    let page = pagination.page();
    let page_size = pagination.page_size();
    let (items, total) = state
        .reviews
        .list_for_creator_paginated(id, page, page_size)
        .await?;
    Ok(Json(ApiResponse::success(PaginatedResponse::new(
        items, total, page, page_size,
    ))))
}
