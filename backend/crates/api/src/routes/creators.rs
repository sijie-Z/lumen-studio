use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use common::{ApiResponse, AppError, PaginatedResponse, Pagination};
use serde::Deserialize;
use services::creator_service::{CreatorProfileDto, UpsertProfileInput};

#[derive(Debug, Deserialize)]
struct ListCreatorsQuery {
    page: Option<u64>,
    page_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct UpsertProfileBody {
    introduction: Option<String>,
    bio: Option<String>,
    service_areas: Option<serde_json::Value>,
    portfolio_url: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/creators", get(list_creators).post(upsert_profile))
        .route("/creators/{id}", get(get_creator))
}

async fn list_creators(
    State(state): State<AppState>,
    Query(query): Query<ListCreatorsQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<CreatorProfileDto>>>, AppError> {
    let pagination = Pagination {
        page: query.page,
        page_size: query.page_size,
    };
    let page = pagination.page();
    let page_size = pagination.page_size();
    let (items, total) = state.creators.list_paginated(page, page_size).await?;
    Ok(Json(ApiResponse::success(PaginatedResponse::new(
        items, total, page, page_size,
    ))))
}

async fn get_creator(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<CreatorProfileDto>>, AppError> {
    let creator = state.creators.by_id(id).await?;
    Ok(Json(ApiResponse::success(creator)))
}

async fn upsert_profile(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<UpsertProfileBody>,
) -> Result<Json<ApiResponse<CreatorProfileDto>>, AppError> {
    let profile = state
        .creators
        .ensure_profile(
            claims.sub,
            UpsertProfileInput {
                introduction: body.introduction,
                bio: body.bio,
                service_areas: body.service_areas,
                portfolio_url: body.portfolio_url,
            },
        )
        .await?;
    Ok(Json(ApiResponse::success(profile)))
}
