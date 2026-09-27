use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, patch},
    Json, Router,
};
use common::{ApiResponse, AppError};
use serde::{Deserialize, Serialize};
use services::notification_service::NotificationDto;

#[derive(Debug, Deserialize)]
struct ListNotificationsQuery {
    limit: Option<u64>,
}

#[derive(Debug, Serialize)]
struct UnreadCountResponse {
    count: u64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/notifications", get(list_notifications))
        .route("/notifications/unread-count", get(unread_count))
        .route("/notifications/read-all", patch(mark_all_read))
        .route("/notifications/{id}/read", patch(mark_read))
}

async fn list_notifications(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<ApiResponse<Vec<NotificationDto>>>, AppError> {
    let notifications = state
        .notifications
        .list_for_user(claims.sub, query.limit)
        .await?;
    Ok(Json(ApiResponse::success(notifications)))
}

async fn unread_count(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<UnreadCountResponse>>, AppError> {
    let count = state.notifications.unread_count(claims.sub).await?;
    Ok(Json(ApiResponse::success(UnreadCountResponse { count })))
}

async fn mark_read(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state.notifications.mark_read(id, claims.sub).await?;
    Ok(Json(ApiResponse::success(())))
}

async fn mark_all_read(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state.notifications.mark_all_read(claims.sub).await?;
    Ok(Json(ApiResponse::success(())))
}
