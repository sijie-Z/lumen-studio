use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use common::{ApiResponse, AppError};
use serde::Deserialize;
use services::work_service::WorkDto;

#[derive(Debug, Deserialize)]
pub struct ListWorksQuery {
    limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkInput {
    image_url: String,
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/works", get(list_works).post(create_work))
}

async fn list_works(
    State(state): State<AppState>,
    Query(query): Query<ListWorksQuery>,
) -> Result<Json<ApiResponse<Vec<WorkDto>>>, AppError> {
    let works = state.works.list(query.limit.unwrap_or(60)).await?;
    Ok(Json(ApiResponse::success(works)))
}

async fn create_work(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(input): Json<CreateWorkInput>,
) -> Result<Json<ApiResponse<WorkDto>>, AppError> {
    let work = state
        .works
        .create(claims.sub, input.image_url, input.title, input.description, input.category)
        .await?;
    Ok(Json(ApiResponse::success(work)))
}
