use chrono::{DateTime, Utc};
use common::{validation::max_chars, AppError};
use db::entities::{
    appointment as appointment_entity, creator_profile as creator_entity, review as review_entity,
};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDto {
    pub id: i32,
    pub appointment_id: i32,
    pub creator_id: i32,
    pub rating: Decimal,
    pub content: Option<String>,
    pub images: Option<serde_json::Value>,
    pub is_anonymous: bool,
    pub photographer_reply: Option<String>,
    pub replied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ReviewService {
    db: DatabaseConnection,
}

impl ReviewService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: i32,
        appointment_id: i32,
        rating: Decimal,
        content: Option<String>,
        is_anonymous: bool,
    ) -> Result<ReviewDto, AppError> {
        if rating < Decimal::ONE || rating > Decimal::from(5) {
            return Err(AppError::BadRequest(
                "rating must be between 1 and 5".into(),
            ));
        }
        max_chars(content.as_deref(), 5000, "review content")?;

        let rating = rating.round_dp(1);
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
        if appointment.status != "completed" {
            return Err(AppError::BadRequest(
                "only completed appointments can be reviewed".into(),
            ));
        }

        let existing = review_entity::Entity::find()
            .filter(review_entity::Column::AppointmentId.eq(appointment_id))
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?;
        if existing.is_some() {
            return Err(AppError::Conflict(
                "appointment has already been reviewed".into(),
            ));
        }

        let review = review_entity::ActiveModel {
            appointment_id: Set(appointment_id),
            user_id: Set(user_id),
            creator_id: Set(appointment.creator_id),
            rating: Set(rating),
            content: Set(content.filter(|value| !value.trim().is_empty())),
            is_anonymous: Set(is_anonymous),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(AppError::from_anyhow)?;

        let creator_reviews = review_entity::Entity::find()
            .filter(review_entity::Column::CreatorId.eq(appointment.creator_id))
            .all(&txn)
            .await
            .map_err(AppError::from_anyhow)?;
        let review_count = creator_reviews.len();
        let rating_total = creator_reviews
            .into_iter()
            .fold(Decimal::ZERO, |total, item| total + item.rating);
        let avg_rating = (rating_total / Decimal::from(review_count as u64)).round_dp(2);

        let creator = creator_entity::Entity::find_by_id(appointment.creator_id)
            .one(&txn)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("creator profile not found".into()))?;
        let mut active_creator = creator.into_active_model();
        active_creator.avg_rating = Set(avg_rating);
        active_creator.updated_at = Set(now);
        active_creator
            .update(&txn)
            .await
            .map_err(AppError::from_anyhow)?;

        txn.commit().await.map_err(AppError::from_anyhow)?;
        Ok(to_dto(review))
    }

    pub async fn list_for_creator(&self, creator_id: i32) -> Result<Vec<ReviewDto>, AppError> {
        self.list_for_creator_paginated(creator_id, 1, u32::MAX as u64)
            .await
            .map(|(items, _)| items)
    }

    pub async fn list_for_creator_paginated(
        &self,
        creator_id: i32,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<ReviewDto>, u64), AppError> {
        let total = review_entity::Entity::find()
            .filter(review_entity::Column::CreatorId.eq(creator_id))
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let reviews = review_entity::Entity::find()
            .filter(review_entity::Column::CreatorId.eq(creator_id))
            .order_by_desc(review_entity::Column::CreatedAt)
            .offset((page.saturating_sub(1)) * page_size)
            .limit(page_size)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok((reviews.into_iter().map(to_dto).collect(), total))
    }
}

fn to_dto(model: review_entity::Model) -> ReviewDto {
    ReviewDto {
        id: model.id,
        appointment_id: model.appointment_id,
        creator_id: model.creator_id,
        rating: model.rating,
        content: model.content,
        images: model.images,
        is_anonymous: model.is_anonymous,
        photographer_reply: model.photographer_reply,
        replied_at: model.replied_at,
        created_at: model.created_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::entities::{creator_profile as creator_entity, user as user_entity};
    use db::migrations;
    use sea_orm::{ActiveModelTrait, Set};

    #[tokio::test]
    async fn creator_reviews_are_paginated_and_content_length_is_limited() {
        let db = db::connect("sqlite::memory:").await.unwrap();
        migrations::run(&db).await.unwrap();
        let now = Utc::now();

        user_entity::ActiveModel {
            username: Set("review_creator".into()),
            password_hash: Set("hash".into()),
            nickname: Set("Review Creator".into()),
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

        for index in 0..3 {
            review_entity::ActiveModel {
                appointment_id: Set(index + 1),
                user_id: Set(1),
                creator_id: Set(1),
                rating: Set(Decimal::new(50, 1)),
                content: Set(Some(format!("review {index}"))),
                is_anonymous: Set(false),
                created_at: Set(now),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();
        }

        let service = ReviewService::new(db);
        let (first, total) = service.list_for_creator_paginated(1, 1, 2).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(first.len(), 2);
        let (second, total) = service.list_for_creator_paginated(1, 2, 2).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(second.len(), 1);

        let too_long = "x".repeat(5001);
        assert!(matches!(
            service
                .create(1, 1, Decimal::new(50, 1), Some(too_long), false)
                .await,
            Err(AppError::BadRequest(_))
        ));
    }
}
