use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use common::{ApiResponse, AppError};
use serde::Deserialize;
use services::favorite_service::{FavoriteDto, FavoriteStatusDto, FavoriteToggleDto};

#[derive(Debug, Deserialize)]
struct ToggleFavoriteInput {
    target_type: String,
    target_id: i32,
}

#[derive(Debug, Deserialize)]
struct ListFavoritesQuery {
    target_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FavoriteStatusQuery {
    target_type: String,
    target_id: i32,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/favorites", get(list_favorites))
        .route("/favorites/status", get(favorite_status))
        .route("/favorites/toggle", post(toggle_favorite))
}

async fn toggle_favorite(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(input): Json<ToggleFavoriteInput>,
) -> Result<Json<ApiResponse<FavoriteToggleDto>>, AppError> {
    let result = state
        .favorites
        .toggle(claims.sub, &input.target_type, input.target_id)
        .await?;
    Ok(Json(ApiResponse::success(result)))
}

async fn list_favorites(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Query(query): Query<ListFavoritesQuery>,
) -> Result<Json<ApiResponse<Vec<FavoriteDto>>>, AppError> {
    let favorites = state
        .favorites
        .list(claims.sub, query.target_type.as_deref())
        .await?;
    Ok(Json(ApiResponse::success(favorites)))
}

async fn favorite_status(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Query(query): Query<FavoriteStatusQuery>,
) -> Result<Json<ApiResponse<FavoriteStatusDto>>, AppError> {
    let status = state
        .favorites
        .status(claims.sub, &query.target_type, query.target_id)
        .await?;
    Ok(Json(ApiResponse::success(status)))
}
