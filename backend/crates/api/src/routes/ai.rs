use crate::state::AppState;
use ai::{ChatRequest, ChatResponse};
use axum::{extract::State, routing::post, Json, Router};
use common::{ApiResponse, AppError};

pub fn router() -> Router<AppState> {
    Router::new().route("/chat", post(chat))
}

async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ApiResponse<ChatResponse>>, AppError> {
    let response = state.chat.chat(&request).await?;
    Ok(Json(ApiResponse::success(response)))
}
