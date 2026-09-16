use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use common::{ApiResponse, AppError};
use serde::Deserialize;
use services::service_catalog::{CreateServiceInput, ServiceDto, ServiceTypeDto};

#[derive(Debug, Deserialize)]
struct ListServicesQuery {
    limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CreateServiceBody {
    type_id: i32,
    title: String,
    description: Option<String>,
    price: rust_decimal::Decimal,
    duration: Option<i32>,
    cover_image_url: Option<String>,
    location: Option<String>,
    tags: Option<String>,
    options: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct SetActiveBody {
    is_active: bool,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/services", get(list_services).post(create_service))
        .route("/service-types", get(list_types))
        .route(
            "/services/{id}",
            get(get_service).patch(set_active),
        )
}

async fn list_types(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ServiceTypeDto>>>, AppError> {
    let types = state.services.list_types().await?;
    Ok(Json(ApiResponse::success(types)))
}

async fn list_services(
    State(state): State<AppState>,
    Query(query): Query<ListServicesQuery>,
) -> Result<Json<ApiResponse<Vec<ServiceDto>>>, AppError> {
    let services = state.services.list_active(query.limit.unwrap_or(60)).await?;
    Ok(Json(ApiResponse::success(services)))
}

async fn get_service(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<ServiceDto>>, AppError> {
    let service = state.services.by_id(id).await?;
    Ok(Json(ApiResponse::success(service)))
}

async fn create_service(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<CreateServiceBody>,
) -> Result<Json<ApiResponse<ServiceDto>>, AppError> {
    let creator = state
        .creators
        .by_user_id(claims.sub)
        .await?
        .ok_or_else(|| AppError::Forbidden("complete creator profile first".into()))?;
    let service = state
        .services
        .create(
            creator.id,
            CreateServiceInput {
                type_id: body.type_id,
                title: body.title,
                description: body.description,
                price: body.price,
                duration: body.duration,
                cover_image_url: body.cover_image_url,
                location: body.location,
                tags: body.tags,
                options: body.options,
            },
        )
        .await?;
    Ok(Json(ApiResponse::success(service)))
}

async fn set_active(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<SetActiveBody>,
) -> Result<Json<ApiResponse<ServiceDto>>, AppError> {
    let creator = state
        .creators
        .by_user_id(claims.sub)
        .await?
        .ok_or_else(|| AppError::Forbidden("complete creator profile first".into()))?;
    let service = state.services.set_active(creator.id, id, body.is_active).await?;
    Ok(Json(ApiResponse::success(service)))
}
