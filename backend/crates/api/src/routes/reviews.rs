use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use common::{ApiResponse, AppError};
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
) -> Result<Json<ApiResponse<Vec<ReviewDto>>>, AppError> {
    let reviews = state.reviews.list_for_creator(id).await?;
    Ok(Json(ApiResponse::success(reviews)))
}
