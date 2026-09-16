use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use common::{ApiResponse, AppError};
use services::admin_service::{AdminUserDto, PlatformStats};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(list_users))
        .route("/admin/stats", get(stats))
}

async fn list_users(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<Vec<AdminUserDto>>>, AppError> {
    require_admin(&claims)?;
    let users = state.admin.list_users().await?;
    Ok(Json(ApiResponse::success(users)))
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
