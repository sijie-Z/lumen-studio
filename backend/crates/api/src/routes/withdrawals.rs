use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::{get, patch},
    Json, Router,
};
use common::{ApiResponse, AppError};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;
use services::withdrawal_service::WithdrawalDto;

#[derive(Debug, Deserialize)]
struct ApplyWithdrawalBody {
    amount: Decimal,
    account_info: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ReviewWithdrawalBody {
    approve: bool,
    note: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/withdrawals",
            get(list_my_withdrawals).post(apply_withdrawal),
        )
        .route("/admin/withdrawals", get(list_all_withdrawals))
        .route("/admin/withdrawals/{id}", patch(review_withdrawal))
}

async fn apply_withdrawal(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<ApplyWithdrawalBody>,
) -> Result<Json<ApiResponse<WithdrawalDto>>, AppError> {
    let withdrawal = state
        .withdrawals
        .apply(claims.sub, body.amount, body.account_info)
        .await?;
    if let Err(error) = state
        .notifications
        .create_for_admins(
            "withdrawal_requested",
            "新的提现申请",
            Some(format!(
                "创作者 #{} 提交了提现申请 #{}，金额 ¥{}。",
                withdrawal.creator_id, withdrawal.id, withdrawal.amount
            )),
            Some("/admin".into()),
        )
        .await
    {
        tracing::warn!(
            ?error,
            withdrawal_id = withdrawal.id,
            "failed to notify admins"
        );
    }
    Ok(Json(ApiResponse::success(withdrawal)))
}

async fn list_my_withdrawals(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<Vec<WithdrawalDto>>>, AppError> {
    let creator = state
        .creators
        .by_user_id(claims.sub)
        .await?
        .ok_or_else(|| AppError::Forbidden("complete creator profile first".into()))?;
    let withdrawals = state.withdrawals.list_for_creator(creator.id).await?;
    Ok(Json(ApiResponse::success(withdrawals)))
}

async fn list_all_withdrawals(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<Vec<WithdrawalDto>>>, AppError> {
    require_admin(&claims)?;
    let withdrawals = state.withdrawals.list_all().await?;
    Ok(Json(ApiResponse::success(withdrawals)))
}

async fn review_withdrawal(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<ReviewWithdrawalBody>,
) -> Result<Json<ApiResponse<WithdrawalDto>>, AppError> {
    require_admin(&claims)?;
    let withdrawal = state
        .withdrawals
        .review(id, claims.sub, body.approve, body.note)
        .await?;
    let title = if body.approve {
        "提现申请已通过"
    } else {
        "提现申请已拒绝"
    };
    if let Err(error) = state
        .notifications
        .create_for_creator(
            withdrawal.creator_id,
            "withdrawal_reviewed",
            title,
            Some(format!(
                "提现申请 #{} 已{}，金额 ¥{}。",
                withdrawal.id,
                if body.approve { "通过" } else { "拒绝" },
                withdrawal.amount
            )),
            Some("/dashboard".into()),
        )
        .await
    {
        tracing::warn!(
            ?error,
            withdrawal_id = withdrawal.id,
            "failed to notify creator"
        );
    }
    Ok(Json(ApiResponse::success(withdrawal)))
}

fn require_admin(claims: &services::auth_service::Claims) -> Result<(), AppError> {
    if claims.role == "admin" {
        Ok(())
    } else {
        Err(AppError::Forbidden("admin access required".into()))
    }
}
