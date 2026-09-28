use chrono::{DateTime, Datelike, Utc};
use common::AppError;
use db::entities::{
    appointment as appointment_entity, creator_profile as creator_entity,
    payment as payment_entity, review as review_entity, withdrawal as withdrawal_entity,
};
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const APPOINTMENT_STATUSES: [&str; 6] = [
    "pending",
    "confirmed",
    "ongoing",
    "completed",
    "cancelled",
    "refunded",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorAnalyticsDto {
    pub total_income: Decimal,
    pub total_appointments: u64,
    pub completed_appointments: u64,
    pub pending_appointments: u64,
    pub completion_rate: Decimal,
    pub avg_rating: Decimal,
    pub review_count: u64,
    pub pending_withdrawal_amount: Decimal,
    pub revenue_by_month: Vec<RevenueByMonthDto>,
    pub appointments_by_status: Vec<AppointmentStatusCountDto>,
    pub rating_distribution: Vec<RatingDistributionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueByMonthDto {
    pub month: String,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentStatusCountDto {
    pub status: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingDistributionDto {
    pub rating: u8,
    pub count: u64,
}

#[derive(Clone)]
pub struct CreatorAnalyticsService {
    db: DatabaseConnection,
}

impl CreatorAnalyticsService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn overview(&self, creator_id: i32) -> Result<CreatorAnalyticsDto, AppError> {
        let creator = creator_entity::Entity::find_by_id(creator_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("creator profile not found".into()))?;

        let appointments = appointment_entity::Entity::find()
            .filter(appointment_entity::Column::CreatorId.eq(creator_id))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let reviews = review_entity::Entity::find()
            .filter(review_entity::Column::CreatorId.eq(creator_id))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let withdrawals = withdrawal_entity::Entity::find()
            .filter(withdrawal_entity::Column::CreatorId.eq(creator_id))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let settlements = payment_entity::Entity::find()
            .filter(payment_entity::Column::UserId.eq(creator.user_id))
            .filter(payment_entity::Column::PaymentType.eq("settlement"))
            .filter(payment_entity::Column::Status.eq("success"))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;

        let total_appointments = appointments.len() as u64;
        let completed_appointments = appointments
            .iter()
            .filter(|item| item.status == "completed")
            .count() as u64;
        let pending_appointments = appointments
            .iter()
            .filter(|item| item.status == "pending")
            .count() as u64;
        let completion_rate = if total_appointments == 0 {
            Decimal::ZERO
        } else {
            (Decimal::from(completed_appointments) / Decimal::from(total_appointments)
                * Decimal::from(100))
            .round_dp(2)
        };

        let pending_withdrawal_amount = withdrawals
            .iter()
            .filter(|item| item.status == "pending")
            .fold(Decimal::ZERO, |total, item| total + item.amount);

        let month_keys = recent_month_keys(Utc::now());
        let mut revenue_by_month = month_keys
            .iter()
            .cloned()
            .map(|month| (month, Decimal::ZERO))
            .collect::<BTreeMap<_, _>>();
        for payment in settlements {
            let key = month_key(payment.created_at);
            if let Some(amount) = revenue_by_month.get_mut(&key) {
                *amount += payment.amount;
            }
        }
        let revenue_by_month = month_keys
            .into_iter()
            .map(|month| RevenueByMonthDto {
                amount: revenue_by_month.remove(&month).unwrap_or(Decimal::ZERO),
                month,
            })
            .collect();

        let appointments_by_status = APPOINTMENT_STATUSES
            .iter()
            .map(|status| AppointmentStatusCountDto {
                status: (*status).to_string(),
                count: appointments
                    .iter()
                    .filter(|item| item.status == *status)
                    .count() as u64,
            })
            .collect();

        let mut rating_buckets = [0_u64; 5];
        for review in &reviews {
            if let Some(index) = rating_bucket(review.rating) {
                rating_buckets[index] += 1;
            }
        }
        let rating_distribution = (1..=5)
            .map(|rating| RatingDistributionDto {
                rating,
                count: rating_buckets[(rating - 1) as usize],
            })
            .collect();

        Ok(CreatorAnalyticsDto {
            total_income: creator.total_income,
            total_appointments,
            completed_appointments,
            pending_appointments,
            completion_rate,
            avg_rating: creator.avg_rating,
            review_count: reviews.len() as u64,
            pending_withdrawal_amount,
            revenue_by_month,
            appointments_by_status,
            rating_distribution,
        })
    }
}

fn month_key(value: DateTime<Utc>) -> String {
    format!("{:04}-{:02}", value.year(), value.month())
}

fn recent_month_keys(now: DateTime<Utc>) -> Vec<String> {
    let current = now.year() as i64 * 12 + now.month0() as i64;
    (0..6)
        .rev()
        .map(|offset| {
            let value = current - offset;
            let year = value.div_euclid(12);
            let month = value.rem_euclid(12) + 1;
            format!("{year:04}-{month:02}")
        })
        .collect()
}

fn rating_bucket(rating: Decimal) -> Option<usize> {
    let rounded = rating.round();
    (1..=5)
        .find(|value| rounded == Decimal::from(*value))
        .map(|value| (value - 1) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use db::entities::{
        appointment as appointment_entity, creator_profile as creator_entity,
        payment as payment_entity, review as review_entity, service as service_entity,
        service_type as service_type_entity, user as user_entity, withdrawal as withdrawal_entity,
    };
    use db::migrations;
    use sea_orm::{ActiveModelTrait, Set};

    async fn insert_user(db: &DatabaseConnection, id: i32, username: &str) {
        let now = Utc::now();
        user_entity::ActiveModel {
            id: Set(id),
            username: Set(username.into()),
            password_hash: Set("hash".into()),
            nickname: Set(username.into()),
            balance: Set(Decimal::ZERO),
            status: Set("active".into()),
            role: Set("user".into()),
            verification_status: Set("unverified".into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();
    }

    async fn insert_appointment(
        db: &DatabaseConnection,
        id: i32,
        status: &str,
        now: DateTime<Utc>,
    ) {
        appointment_entity::ActiveModel {
            id: Set(id),
            user_id: Set(2),
            creator_id: Set(1),
            service_id: Set(1),
            appointment_date: Set(now.format("%Y-%m-%d").to_string()),
            start_time: Set(now),
            end_time: Set(now + chrono::Duration::hours(2)),
            status: Set(status.into()),
            total_price: Set(Decimal::new(10000, 2)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn creator_analytics_aggregates_income_appointments_and_ratings() {
        let db = db::connect("sqlite::memory:").await.unwrap();
        migrations::run(&db).await.unwrap();

        let now = Utc::now();
        insert_user(&db, 1, "analytics_creator").await;
        insert_user(&db, 2, "analytics_client").await;

        creator_entity::ActiveModel {
            id: Set(1),
            user_id: Set(1),
            rating: Set(Decimal::new(50, 1)),
            certification_level: Set("standard".into()),
            total_services: Set(1),
            total_appointments: Set(3),
            total_income: Set(Decimal::new(18000, 2)),
            avg_rating: Set(Decimal::new(450, 2)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        service_type_entity::ActiveModel {
            id: Set(1),
            name: Set("portrait".into()),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        service_entity::ActiveModel {
            id: Set(1),
            creator_id: Set(1),
            type_id: Set(1),
            title: Set("Portrait".into()),
            price: Set(Decimal::new(10000, 2)),
            duration: Set(Some(120)),
            is_active: Set(true),
            is_featured: Set(false),
            appointments_count: Set(3),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        insert_appointment(&db, 1, "completed", now).await;
        insert_appointment(&db, 2, "confirmed", now).await;
        insert_appointment(&db, 3, "pending", now).await;

        let previous_month = Utc
            .with_ymd_and_hms(now.year(), now.month(), 1, 12, 0, 0)
            .single()
            .unwrap()
            - chrono::Duration::days(1);
        payment_entity::ActiveModel {
            appointment_id: Set(Some(1)),
            user_id: Set(1),
            amount: Set(Decimal::new(10000, 2)),
            status: Set("success".into()),
            payment_type: Set("settlement".into()),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        payment_entity::ActiveModel {
            appointment_id: Set(Some(2)),
            user_id: Set(1),
            amount: Set(Decimal::new(8000, 2)),
            status: Set("success".into()),
            payment_type: Set("settlement".into()),
            created_at: Set(previous_month),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        payment_entity::ActiveModel {
            user_id: Set(1),
            amount: Set(Decimal::new(50000, 2)),
            status: Set("success".into()),
            payment_type: Set("recharge".into()),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        review_entity::ActiveModel {
            appointment_id: Set(1),
            user_id: Set(2),
            creator_id: Set(1),
            rating: Set(Decimal::new(50, 1)),
            is_anonymous: Set(false),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        review_entity::ActiveModel {
            appointment_id: Set(2),
            user_id: Set(2),
            creator_id: Set(1),
            rating: Set(Decimal::new(40, 1)),
            is_anonymous: Set(false),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        withdrawal_entity::ActiveModel {
            creator_id: Set(1),
            amount: Set(Decimal::new(3000, 2)),
            fee: Set(Decimal::ZERO),
            actual_amount: Set(Some(Decimal::new(3000, 2))),
            status: Set("pending".into()),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        withdrawal_entity::ActiveModel {
            creator_id: Set(1),
            amount: Set(Decimal::new(2000, 2)),
            fee: Set(Decimal::ZERO),
            actual_amount: Set(Some(Decimal::new(2000, 2))),
            status: Set("completed".into()),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let analytics = CreatorAnalyticsService::new(db).overview(1).await.unwrap();

        assert_eq!(analytics.total_income, Decimal::new(18000, 2));
        assert_eq!(analytics.total_appointments, 3);
        assert_eq!(analytics.completed_appointments, 1);
        assert_eq!(analytics.pending_appointments, 1);
        assert_eq!(analytics.completion_rate, Decimal::new(3333, 2));
        assert_eq!(analytics.avg_rating, Decimal::new(450, 2));
        assert_eq!(analytics.review_count, 2);
        assert_eq!(analytics.pending_withdrawal_amount, Decimal::new(3000, 2));
        assert_eq!(analytics.revenue_by_month.len(), 6);
        let revenue_total = analytics
            .revenue_by_month
            .iter()
            .fold(Decimal::ZERO, |total, item| total + item.amount);
        assert_eq!(revenue_total, Decimal::new(18000, 2));
        assert_eq!(
            analytics
                .appointments_by_status
                .iter()
                .find(|item| item.status == "completed")
                .unwrap()
                .count,
            1
        );
        assert_eq!(
            analytics
                .rating_distribution
                .iter()
                .find(|item| item.rating == 5)
                .unwrap()
                .count,
            1
        );
        assert_eq!(
            analytics
                .rating_distribution
                .iter()
                .find(|item| item.rating == 4)
                .unwrap()
                .count,
            1
        );
    }
}
