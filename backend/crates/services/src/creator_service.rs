use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{creator_profile as profile_entity, CreatorProfileModel};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CreatorProfileDto {
    pub id: i32,
    pub user_id: i32,
    pub introduction: Option<String>,
    pub bio: Option<String>,
    pub rating: Decimal,
    pub certification_level: String,
    pub service_areas: Option<serde_json::Value>,
    pub style_vector_id: Option<String>,
    pub portfolio_url: Option<String>,
    pub total_services: i32,
    pub total_appointments: i32,
    pub total_income: Decimal,
    pub avg_rating: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UpsertProfileInput {
    pub introduction: Option<String>,
    pub bio: Option<String>,
    pub service_areas: Option<serde_json::Value>,
    pub portfolio_url: Option<String>,
}

#[derive(Clone)]
pub struct CreatorService {
    db: DatabaseConnection,
}

impl CreatorService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn ensure_profile(
        &self,
        user_id: i32,
        input: UpsertProfileInput,
    ) -> Result<CreatorProfileDto, AppError> {
        let existing = profile_entity::Entity::find()
            .filter(profile_entity::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;

        if let Some(model) = existing {
            let mut active = model.clone().into_active_model();
            active.introduction = Set(clean(input.introduction));
            active.bio = Set(clean(input.bio));
            active.service_areas = Set(input.service_areas);
            active.portfolio_url = Set(clean(input.portfolio_url));
            active.updated_at = Set(Utc::now());
            let updated = active
                .update(&self.db)
                .await
                .map_err(AppError::from_anyhow)?;
            return Ok(to_dto(updated));
        }

        let now = Utc::now();
        let model = profile_entity::ActiveModel {
            user_id: Set(user_id),
            introduction: Set(clean(input.introduction)),
            bio: Set(clean(input.bio)),
            rating: Set(Decimal::new(50, 1)),
            certification_level: Set("standard".into()),
            service_areas: Set(input.service_areas),
            available_slots: Set(None),
            style_vector_id: Set(None),
            portfolio_url: Set(clean(input.portfolio_url)),
            total_services: Set(0),
            total_appointments: Set(0),
            total_income: Set(Decimal::ZERO),
            avg_rating: Set(Decimal::ZERO),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(AppError::from_anyhow)?;

        Ok(to_dto(model))
    }

    pub async fn by_user_id(&self, user_id: i32) -> Result<Option<CreatorProfileDto>, AppError> {
        let model = profile_entity::Entity::find()
            .filter(profile_entity::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(model.map(to_dto))
    }

    pub async fn by_id(&self, id: i32) -> Result<CreatorProfileDto, AppError> {
        let model = profile_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("creator not found".into()))?;
        Ok(to_dto(model))
    }

    pub async fn list(&self, limit: u64) -> Result<Vec<CreatorProfileDto>, AppError> {
        let models = profile_entity::Entity::find()
            .order_by_desc(profile_entity::Column::Rating)
            .limit(limit.min(100))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(models.into_iter().map(to_dto).collect())
    }

    pub async fn increment_stats(
        &self,
        creator_id: i32,
        services_delta: i32,
        appointments_delta: i32,
        income: Decimal,
    ) -> Result<(), AppError> {
        let model = profile_entity::Entity::find_by_id(creator_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("creator not found".into()))?;
        let total_services = model.total_services;
        let total_appointments = model.total_appointments;
        let total_income = model.total_income;
        let mut active = model.into_active_model();
        active.total_services = Set(total_services + services_delta);
        active.total_appointments = Set(total_appointments + appointments_delta);
        active.total_income = Set(total_income + income);
        active.updated_at = Set(Utc::now());
        active
            .update(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(())
    }
}

fn clean(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty())
}

fn to_dto(model: CreatorProfileModel) -> CreatorProfileDto {
    CreatorProfileDto {
        id: model.id,
        user_id: model.user_id,
        introduction: model.introduction,
        bio: model.bio,
        rating: model.rating,
        certification_level: model.certification_level,
        service_areas: model.service_areas,
        style_vector_id: model.style_vector_id,
        portfolio_url: model.portfolio_url,
        total_services: model.total_services,
        total_appointments: model.total_appointments,
        total_income: model.total_income,
        avg_rating: model.avg_rating,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}
