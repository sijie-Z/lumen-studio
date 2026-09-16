use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::{get, patch},
    Json, Router,
};
use common::{ApiResponse, AppError};
use serde::Deserialize;
use services::appointment_service::{AppointmentDto, CreateAppointmentInput};

#[derive(Debug, Deserialize)]
struct TransitionBody {
    status: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/appointments", get(list_appointments).post(create_appointment))
        .route("/appointments/{id}", patch(transition))
}

async fn list_appointments(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<Vec<AppointmentDto>>>, AppError> {
    let appointments = state.appointments.list_for_user(claims.sub).await?;
    Ok(Json(ApiResponse::success(appointments)))
}

async fn create_appointment(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<CreateAppointmentInput>,
) -> Result<Json<ApiResponse<AppointmentDto>>, AppError> {
    let appointment = state.appointments.create(claims.sub, body).await?;
    Ok(Json(ApiResponse::success(appointment)))
}

async fn transition(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<TransitionBody>,
) -> Result<Json<ApiResponse<AppointmentDto>>, AppError> {
    let creator_id = state
        .creators
        .by_user_id(claims.sub)
        .await?
        .map(|profile| profile.id);
    let appointment = state
        .appointments
        .transition(claims.sub, creator_id, id, body.status)
        .await?;
    Ok(Json(ApiResponse::success(appointment)))
}
