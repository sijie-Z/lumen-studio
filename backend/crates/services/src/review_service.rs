use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{
    appointment as appointment_entity, creator_profile as creator_entity, review as review_entity,
};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set, TransactionTrait,
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
        let reviews = review_entity::Entity::find()
            .filter(review_entity::Column::CreatorId.eq(creator_id))
            .order_by_desc(review_entity::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(reviews.into_iter().map(to_dto).collect())
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
