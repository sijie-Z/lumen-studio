use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{
    creator_profile as creator_entity, user as user_entity, withdrawal as withdrawal_entity,
};
use rust_decimal::Decimal;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalDto {
    pub id: i32,
    pub creator_id: i32,
    pub amount: Decimal,
    pub fee: Decimal,
    pub actual_amount: Option<Decimal>,
    pub status: String,
    pub account_info: Option<Value>,
    pub reviewed_by: Option<i32>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct WithdrawalService {
    db: DatabaseConnection,
}

impl WithdrawalService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn apply(
        &self,
        creator_user_id: i32,
        amount: Decimal,
        account_info: Option<Value>,
    ) -> Result<WithdrawalDto, AppError> {
        let amount = positive_money(amount, "withdrawal amount")?;
        let now = Utc::now();
        let txn = self.db.begin().await.map_err(map_withdrawal_db_error)?;

        let creator = creator_entity::Entity::find()
            .filter(creator_entity::Column::UserId.eq(creator_user_id))
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::Forbidden("creator profile required".into()))?;

        let user = user_entity::Entity::find_by_id(creator_user_id)
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("creator user not found".into()))?;
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

        let withdrawal = withdrawal_entity::ActiveModel {
            creator_id: Set(creator.id),
            amount: Set(amount.amount()),
            fee: Set(Decimal::ZERO),
            actual_amount: Set(Some(amount.amount())),
            status: Set("pending".into()),
            account_info: Set(account_info),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(AppError::from_anyhow)?;

        txn.commit().await.map_err(AppError::from_anyhow)?;
        Ok(to_dto(withdrawal))
    }

    pub async fn list_for_creator(&self, creator_id: i32) -> Result<Vec<WithdrawalDto>, AppError> {
        let withdrawals = withdrawal_entity::Entity::find()
            .filter(withdrawal_entity::Column::CreatorId.eq(creator_id))
            .order_by_desc(withdrawal_entity::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(withdrawals.into_iter().map(to_dto).collect())
    }

    pub async fn list_all(&self) -> Result<Vec<WithdrawalDto>, AppError> {
        self.list_all_paginated(1, u32::MAX as u64)
            .await
            .map(|(items, _)| items)
    }

    pub async fn list_all_paginated(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<WithdrawalDto>, u64), AppError> {
        let total = withdrawal_entity::Entity::find()
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let withdrawals = withdrawal_entity::Entity::find()
            .order_by_desc(withdrawal_entity::Column::CreatedAt)
            .offset((page.saturating_sub(1)) * page_size)
            .limit(page_size)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok((withdrawals.into_iter().map(to_dto).collect(), total))
    }

    pub async fn review(
        &self,
        withdrawal_id: i32,
        admin_user_id: i32,
        approve: bool,
        note: Option<String>,
    ) -> Result<WithdrawalDto, AppError> {
        let now = Utc::now();
        let txn = self.db.begin().await.map_err(map_withdrawal_db_error)?;

        let withdrawal = if txn.get_database_backend() == DbBackend::Postgres {
            withdrawal_entity::Entity::find_by_id(withdrawal_id)
                .lock_exclusive()
                .one(&txn)
                .await
        } else {
            withdrawal_entity::Entity::find_by_id(withdrawal_id)
                .one(&txn)
                .await
        }
        .map_err(map_withdrawal_db_error)?
        .ok_or_else(|| AppError::NotFound("withdrawal not found".into()))?;
        if withdrawal.status != "pending" {
            return Err(AppError::Conflict(
                "withdrawal has already been reviewed".into(),
            ));
        }

        let creator_id = withdrawal.creator_id;
        let withdrawal_amount = withdrawal.amount;
        let (status, completed_at) = if approve {
            ("completed", Some(now))
        } else {
            let creator = creator_entity::Entity::find_by_id(creator_id)
                .one(&txn)
                .await
                .map_err(map_withdrawal_db_error)?
                .ok_or_else(|| AppError::NotFound("creator profile not found".into()))?;
            let user = user_entity::Entity::find_by_id(creator.user_id)
                .one(&txn)
                .await
                .map_err(map_withdrawal_db_error)?
                .ok_or_else(|| AppError::NotFound("creator user not found".into()))?;

            let amount = positive_money(withdrawal_amount, "withdrawal amount")?;
            let new_balance = user.balance + amount.amount();
            let mut active_user = user.into_active_model();
            active_user.balance = Set(new_balance);
            active_user.updated_at = Set(now);
            active_user
                .update(&txn)
                .await
                .map_err(map_withdrawal_db_error)?;
            ("rejected", None)
        };

        let updated = withdrawal_entity::Entity::update_many()
            .col_expr(
                withdrawal_entity::Column::ReviewedBy,
                Expr::value(admin_user_id),
            )
            .col_expr(
                withdrawal_entity::Column::ReviewedAt,
                Expr::value(Some(now)),
            )
            .col_expr(withdrawal_entity::Column::ReviewNote, Expr::value(note))
            .col_expr(withdrawal_entity::Column::Status, Expr::value(status))
            .col_expr(
                withdrawal_entity::Column::CompletedAt,
                Expr::value(completed_at),
            )
            .filter(withdrawal_entity::Column::Id.eq(withdrawal_id))
            .filter(withdrawal_entity::Column::Status.eq("pending"))
            .exec(&txn)
            .await
            .map_err(map_withdrawal_db_error)?;
        if updated.rows_affected != 1 {
            return Err(AppError::Conflict(
                "withdrawal has already been reviewed".into(),
            ));
        }

        let withdrawal = withdrawal_entity::Entity::find_by_id(withdrawal_id)
            .one(&txn)
            .await
            .map_err(map_withdrawal_db_error)?
            .ok_or_else(|| AppError::NotFound("withdrawal not found".into()))?;
        txn.commit().await.map_err(map_withdrawal_db_error)?;
        Ok(to_dto(withdrawal))
    }
}

fn map_withdrawal_db_error(error: sea_orm::DbErr) -> AppError {
    let message = error.to_string().to_lowercase();
    if message.contains("database is locked")
        || message.contains("database table is locked")
        || message.contains("database is busy")
        || message.contains("deadlock")
    {
        AppError::Conflict("withdrawal review is already in progress; retry".into())
    } else {
        AppError::Internal(anyhow::Error::new(error))
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

fn to_dto(model: withdrawal_entity::Model) -> WithdrawalDto {
    WithdrawalDto {
        id: model.id,
        creator_id: model.creator_id,
        amount: model.amount,
        fee: model.fee,
        actual_amount: model.actual_amount,
        status: model.status,
        account_info: model.account_info,
        reviewed_by: model.reviewed_by,
        reviewed_at: model.reviewed_at,
        review_note: model.review_note,
        completed_at: model.completed_at,
        created_at: model.created_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::migrations;

    #[tokio::test]
    async fn admin_withdrawals_are_paginated() {
        let db = db::connect("sqlite::memory:").await.unwrap();
        migrations::run(&db).await.unwrap();
        let now = Utc::now();

        user_entity::ActiveModel {
            username: Set("withdrawal_page_creator".into()),
            password_hash: Set("hash".into()),
            nickname: Set("Withdrawal Page Creator".into()),
            status: Set("active".into()),
            role: Set("user".into()),
            verification_status: Set("unverified".into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        creator_entity::ActiveModel {
            user_id: Set(1),
            rating: Set(Decimal::new(50, 1)),
            certification_level: Set("standard".into()),
            total_services: Set(0),
            total_appointments: Set(0),
            total_income: Set(Decimal::ZERO),
            avg_rating: Set(Decimal::ZERO),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        for amount in ["10.00", "20.00", "30.00"] {
            withdrawal_entity::ActiveModel {
                creator_id: Set(1),
                amount: Set(amount.parse().unwrap()),
                fee: Set(Decimal::ZERO),
                actual_amount: Set(Some(amount.parse().unwrap())),
                status: Set("pending".into()),
                created_at: Set(now),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();
        }

        let service = WithdrawalService::new(db);
        let (first, total) = service.list_all_paginated(1, 2).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(first.len(), 2);
        let (second, total) = service.list_all_paginated(2, 2).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(second.len(), 1);
    }
}
