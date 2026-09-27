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
        .route(
            "/appointments",
            get(list_appointments).post(create_appointment),
        )
        .route("/appointments/creator", get(list_creator_appointments))
        .route("/appointments/{id}", patch(transition))
}

async fn list_appointments(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<Vec<AppointmentDto>>>, AppError> {
    let appointments = state.appointments.list_for_user(claims.sub).await?;
    Ok(Json(ApiResponse::success(appointments)))
}

async fn list_creator_appointments(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<Vec<AppointmentDto>>>, AppError> {
    let creator = state
        .creators
        .by_user_id(claims.sub)
        .await?
        .ok_or_else(|| AppError::Forbidden("complete creator profile first".into()))?;
    let appointments = state.appointments.list_for_creator(creator.id).await?;
    Ok(Json(ApiResponse::success(appointments)))
}

async fn create_appointment(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<CreateAppointmentInput>,
) -> Result<Json<ApiResponse<AppointmentDto>>, AppError> {
    let appointment = state.appointments.create(claims.sub, body).await?;
    if let Err(error) = state
        .notifications
        .create_for_creator(
            appointment.creator_id,
            "appointment_created",
            "新的预约请求",
            Some(format!(
                "客户提交了预约 #{}，时间：{}。",
                appointment.id, appointment.start_time
            )),
            Some("/dashboard".into()),
        )
        .await
    {
        tracing::warn!(
            ?error,
            appointment_id = appointment.id,
            "failed to create appointment notification"
        );
    }
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
    let target = body.status;
    let appointment = state
        .appointments
        .transition(claims.sub, creator_id, id, target.clone())
        .await?;
    if target == "completed" {
        if let Err(error) = state
            .notifications
            .create(
                appointment.user_id,
                "appointment_completed",
                "服务已完成，欢迎评价",
                Some(format!("预约 #{} 已完成，可以前往订单页提交评价。", id)),
                Some("/account".into()),
            )
            .await
        {
            tracing::warn!(
                ?error,
                appointment_id = id,
                "failed to create completion notification"
            );
        }
    }
    Ok(Json(ApiResponse::success(appointment)))
}
