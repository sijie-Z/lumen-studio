use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{
    appointment as appt_entity, creator_profile as profile_entity, payment as payment_entity,
    service as service_entity, user as user_entity, work as work_entity,
};
use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AdminUserDto {
    pub id: i32,
    pub username: String,
    pub nickname: String,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformStats {
    pub users: u64,
    pub creators: u64,
    pub services: u64,
    pub appointments: u64,
    pub works: u64,
    pub gross_volume: Decimal,
}

#[derive(Clone)]
pub struct AdminService {
    db: DatabaseConnection,
}

impl AdminService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list_users(&self) -> Result<Vec<AdminUserDto>, AppError> {
        self.list_users_paginated(1, u32::MAX as u64)
            .await
            .map(|(items, _)| items)
    }

    pub async fn list_users_paginated(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<AdminUserDto>, u64), AppError> {
        let total = user_entity::Entity::find()
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let users = user_entity::Entity::find()
            .order_by_desc(user_entity::Column::CreatedAt)
            .offset((page.saturating_sub(1)) * page_size)
            .limit(page_size)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok((
            users
                .into_iter()
                .map(|u| AdminUserDto {
                    id: u.id,
                    username: u.username,
                    nickname: u.nickname,
                    role: u.role,
                    status: u.status,
                    created_at: u.created_at,
                })
                .collect(),
            total,
        ))
    }

    pub async fn stats(&self) -> Result<PlatformStats, AppError> {
        let users = user_entity::Entity::find()
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let creators = profile_entity::Entity::find()
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let services = service_entity::Entity::find()
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let appointments = appt_entity::Entity::find()
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let works = work_entity::Entity::find()
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        // Gross volume follows successful appointment payments, excluding recharges,
        // failed payments, refunds, and creator settlement records.
        let gross_volume = payment_entity::Entity::find()
            .filter(
                payment_entity::Column::PaymentType
                    .eq("appointment")
                    .and(payment_entity::Column::Status.eq("success")),
            )
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .into_iter()
            .fold(Decimal::ZERO, |acc, item| acc + item.amount);

        Ok(PlatformStats {
            users,
            creators,
            services,
            appointments,
            works,
            gross_volume,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use db::migrations;
    use sea_orm::{ActiveModelTrait, Set};

    #[tokio::test]
    async fn stats_use_successful_appointment_payments_and_users_are_paginated() {
        let db = db::connect("sqlite::memory:").await.unwrap();
        migrations::run(&db).await.unwrap();

        for index in 0..3 {
            let now = Utc::now();
            user_entity::ActiveModel {
                username: Set(format!("admin_stats_user_{index}")),
                password_hash: Set("hash".into()),
                nickname: Set(format!("Stats User {index}")),
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
        }

        for (amount, status, payment_type) in [
            ("100.00", "success", "appointment"),
            ("999.00", "success", "recharge"),
            ("555.00", "failed", "appointment"),
            ("200.00", "success", "settlement"),
        ] {
            payment_entity::ActiveModel {
                user_id: Set(1),
                amount: Set(amount.parse().unwrap()),
                status: Set(status.into()),
                payment_type: Set(payment_type.into()),
                created_at: Set(Utc::now()),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();
        }

        let service = AdminService::new(db);
        let stats = service.stats().await.unwrap();
        assert_eq!(stats.gross_volume, Decimal::new(100_00, 2));

        let (first, total) = service.list_users_paginated(1, 2).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(first.len(), 2);
        let (second, total) = service.list_users_paginated(2, 2).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(second.len(), 1);
    }
}
