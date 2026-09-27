use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{
    appointment as appointment_entity, creator_profile as creator_entity,
    payment as payment_entity, user as user_entity,
};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentDto {
    pub id: i32,
    pub appointment_id: Option<i32>,
    pub user_id: i32,
    pub amount: Decimal,
    pub method: Option<String>,
    pub status: String,
    pub payment_type: String,
    pub tx_id: Option<String>,
    pub expire_time: Option<DateTime<Utc>>,
    pub payment_channel: Option<String>,
    pub refund_amount: Option<Decimal>,
    pub refund_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PaymentService {
    db: DatabaseConnection,
}

impl PaymentService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn recharge(&self, user_id: i32, amount: Decimal) -> Result<PaymentDto, AppError> {
        let amount = positive_money(amount, "recharge amount")?;
        let now = Utc::now();
        let txn = self.db.begin().await.map_err(AppError::from_anyhow)?;

        let user = user_entity::Entity::find_by_id(user_id)
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;
        let new_balance = user.balance + amount.amount();

        let mut active_user = user.into_active_model();
        active_user.balance = Set(new_balance);
        active_user.updated_at = Set(now);
        active_user
            .update(&txn)
            .await
            .map_err(AppError::from_anyhow)?;

        let payment = payment_entity::ActiveModel {
            user_id: Set(user_id),
            amount: Set(amount.amount()),
            method: Set(Some("balance".into())),
            status: Set("success".into()),
            payment_type: Set("recharge".into()),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(AppError::from_anyhow)?;

        txn.commit().await.map_err(AppError::from_anyhow)?;
        Ok(to_dto(payment))
    }

    pub async fn pay_appointment(
        &self,
        user_id: i32,
        appointment_id: i32,
        method: String,
    ) -> Result<PaymentDto, AppError> {
        let now = Utc::now();
        let txn = self.db.begin().await.map_err(AppError::from_anyhow)?;

        let appointment = appointment_entity::Entity::find_by_id(appointment_id)
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("appointment not found".into()))?;

        if appointment.user_id != user_id {
            return Err(AppError::Forbidden(
                "appointment does not belong to this user".into(),
            ));
        }
        if appointment.status != "pending" {
            return Err(AppError::Conflict(
                "only pending appointments can be paid".into(),
            ));
        }

        let already_paid = payment_entity::Entity::find()
            .filter(
                payment_entity::Column::AppointmentId
                    .eq(appointment_id)
                    .and(payment_entity::Column::PaymentType.eq("appointment"))
                    .and(payment_entity::Column::Status.eq("success")),
            )
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?;
        if already_paid.is_some() {
            return Err(AppError::Conflict(
                "appointment has already been paid".into(),
            ));
        }

        let amount = positive_money(appointment.total_price, "appointment amount")?;
        let user = user_entity::Entity::find_by_id(user_id)
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;
        if user.balance < amount.amount() {
            return Err(AppError::BadRequest("insufficient balance".into()));
        }
        let new_balance = user.balance - amount.amount();

        let mut active_user = user.into_active_model();
        active_user.balance = Set(new_balance);
        active_user.updated_at = Set(now);
        active_user
            .update(&txn)
            .await
            .map_err(AppError::from_anyhow)?;

        let mut active_appointment = appointment.into_active_model();
        active_appointment.status = Set("confirmed".into());
        active_appointment.updated_at = Set(now);
        active_appointment
            .update(&txn)
            .await
            .map_err(AppError::from_anyhow)?;

        let payment = payment_entity::ActiveModel {
            appointment_id: Set(Some(appointment_id)),
            user_id: Set(user_id),
            amount: Set(amount.amount()),
            method: Set(Some(method)),
            status: Set("success".into()),
            payment_type: Set("appointment".into()),
            payment_channel: Set(Some("escrow".into())),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(AppError::from_anyhow)?;

        txn.commit().await.map_err(AppError::from_anyhow)?;
        Ok(to_dto(payment))
    }

    pub async fn settle(&self, appointment_id: i32) -> Result<(), AppError> {
        let now = Utc::now();
        let txn = self.db.begin().await.map_err(AppError::from_anyhow)?;

        let appointment = appointment_entity::Entity::find_by_id(appointment_id)
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("appointment not found".into()))?;

        if appointment.status != "completed" {
            return Err(AppError::BadRequest(
                "appointment must be completed before settlement".into(),
            ));
        }

        let already_settled = payment_entity::Entity::find()
            .filter(
                payment_entity::Column::AppointmentId
                    .eq(appointment_id)
                    .and(payment_entity::Column::PaymentType.eq("settlement")),
            )
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?;
        if already_settled.is_some() {
            return Ok(());
        }

        let paid_payment = payment_entity::Entity::find()
            .filter(
                payment_entity::Column::AppointmentId
                    .eq(appointment_id)
                    .and(payment_entity::Column::PaymentType.eq("appointment"))
                    .and(payment_entity::Column::Status.eq("success")),
            )
            .order_by_desc(payment_entity::Column::CreatedAt)
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::BadRequest("appointment has no successful payment".into()))?;

        let paid_amount = positive_money(paid_payment.amount, "paid amount")?;
        let commission = paid_amount.commission();
        let creator_amount = paid_amount.amount() - commission.amount();
        if creator_amount < Decimal::ZERO {
            return Err(AppError::Internal(anyhow::anyhow!(
                "invalid commission calculation"
            )));
        }

        let creator = creator_entity::Entity::find_by_id(appointment.creator_id)
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("creator profile not found".into()))?;
        let creator_user_id = creator.user_id;

        let creator_user = user_entity::Entity::find_by_id(creator_user_id)
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("creator user not found".into()))?;
        let creator_balance = creator_user.balance + creator_amount;

        let mut active_user = creator_user.into_active_model();
        active_user.balance = Set(creator_balance);
        active_user.updated_at = Set(now);
        active_user
            .update(&txn)
            .await
            .map_err(AppError::from_anyhow)?;

        let creator_income = creator.total_income + creator_amount;
        let mut active_creator = creator.into_active_model();
        active_creator.total_income = Set(creator_income);
        active_creator.updated_at = Set(now);
        active_creator
            .update(&txn)
            .await
            .map_err(AppError::from_anyhow)?;

        payment_entity::ActiveModel {
            appointment_id: Set(Some(appointment_id)),
            user_id: Set(creator_user_id),
            amount: Set(creator_amount),
            status: Set("success".into()),
            payment_type: Set("settlement".into()),
            payment_channel: Set(Some("settlement".into())),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(AppError::from_anyhow)?;

        txn.commit().await.map_err(AppError::from_anyhow)?;
        Ok(())
    }

    pub async fn list_for_user(&self, user_id: i32) -> Result<Vec<PaymentDto>, AppError> {
        let payments = payment_entity::Entity::find()
            .filter(payment_entity::Column::UserId.eq(user_id))
            .order_by_desc(payment_entity::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(payments.into_iter().map(to_dto).collect())
    }
}

fn positive_money(amount: Decimal, label: &str) -> Result<app_core::Money, AppError> {
    if amount <= Decimal::ZERO {
        return Err(AppError::BadRequest(format!(
            "{label} must be greater than zero"
        )));
    }
    app_core::Money::new(amount).map_err(|err| AppError::BadRequest(err.to_string()))
}

fn to_dto(model: payment_entity::Model) -> PaymentDto {
    PaymentDto {
        id: model.id,
        appointment_id: model.appointment_id,
        user_id: model.user_id,
        amount: model.amount,
        method: model.method,
        status: model.status,
        payment_type: model.payment_type,
        tx_id: model.tx_id,
        expire_time: model.expire_time,
        payment_channel: model.payment_channel,
        refund_amount: model.refund_amount,
        refund_reason: model.refund_reason,
        created_at: model.created_at,
    }
}
