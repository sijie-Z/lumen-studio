use crate::state::AppState;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use common::{ApiResponse, AppError};
use services::dto::{AuthResponse, LoginInput, RegisterInput, UserDto};
use crate::middleware::auth::AuthUser;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me))
}

async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterInput>,
) -> Result<Json<ApiResponse<UserDto>>, AppError> {
    let user = state.auth.register(input).await?;
    Ok(Json(ApiResponse::success(user)))
}

async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let response = state.auth.login(input).await?;
    Ok(Json(ApiResponse::success(response)))
}

async fn me(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<UserDto>>, AppError> {
    let user = state.auth.get_user_by_id(claims.sub).await?;
    Ok(Json(ApiResponse::success(user)))
}
