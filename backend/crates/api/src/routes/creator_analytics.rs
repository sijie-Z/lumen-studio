use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use common::{ApiResponse, AppError};
use services::creator_analytics_service::CreatorAnalyticsDto;

pub fn router() -> Router<AppState> {
    Router::new().route("/creator/analytics", get(get_creator_analytics))
}

async fn get_creator_analytics(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<CreatorAnalyticsDto>>, AppError> {
    let creator = state
        .creators
        .by_user_id(claims.sub)
        .await?
        .ok_or_else(|| AppError::Forbidden("complete creator profile first".into()))?;
    let analytics = state.analytics.overview(creator.id).await?;
    Ok(Json(ApiResponse::success(analytics)))
}
