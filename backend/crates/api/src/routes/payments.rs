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
use services::payment_service::PaymentDto;

#[derive(Debug, Deserialize)]
struct RechargeBody {
    amount: Decimal,
}

#[derive(Debug, Deserialize)]
struct PayAppointmentBody {
    method: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/payments", get(list_payments))
        .route("/payments/recharge", post(recharge))
        .route("/payments/appointments/{id}/pay", post(pay_appointment))
}

async fn list_payments(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<Vec<PaymentDto>>>, AppError> {
    let payments = state.payments.list_for_user(claims.sub).await?;
    Ok(Json(ApiResponse::success(payments)))
}

async fn recharge(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<RechargeBody>,
) -> Result<Json<ApiResponse<PaymentDto>>, AppError> {
    let payment = state.payments.recharge(claims.sub, body.amount).await?;
    Ok(Json(ApiResponse::success(payment)))
}

async fn pay_appointment(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<PayAppointmentBody>,
) -> Result<Json<ApiResponse<PaymentDto>>, AppError> {
    let method = body.method.unwrap_or_else(|| "balance".into());
    let payment = state
        .payments
        .pay_appointment(claims.sub, id, method)
        .await?;
    Ok(Json(ApiResponse::success(payment)))
}
