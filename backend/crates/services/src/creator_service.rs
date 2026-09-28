use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{creator_profile as profile_entity, user as user_entity, CreatorProfileModel};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CreatorProfileDto {
    pub id: i32,
    pub user_id: i32,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub introduction: Option<String>,
    pub bio: Option<String>,
    pub rating: Decimal,
    pub certification_level: String,
    pub service_areas: Option<serde_json::Value>,
    pub style_vector_id: Option<String>,
    pub portfolio_url: Option<String>,
    pub total_services: i32,
    pub total_appointments: i32,
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
            return self.to_dto_with_identity(updated).await;
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

        self.to_dto_with_identity(model).await
    }

    pub async fn by_user_id(&self, user_id: i32) -> Result<Option<CreatorProfileDto>, AppError> {
        let rows = profile_entity::Entity::find()
            .filter(profile_entity::Column::UserId.eq(user_id))
            .find_with_related(user_entity::Entity)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(rows
            .into_iter()
            .next()
            .map(|(model, users)| dto_from_related(model, &users)))
    }

    pub async fn by_id(&self, id: i32) -> Result<CreatorProfileDto, AppError> {
        let rows = profile_entity::Entity::find_by_id(id)
            .find_with_related(user_entity::Entity)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let (model, users) = rows
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("creator not found".into()))?;
        Ok(dto_from_related(model, &users))
    }

    pub async fn list(&self, limit: u64) -> Result<Vec<CreatorProfileDto>, AppError> {
        let rows = profile_entity::Entity::find()
            .find_with_related(user_entity::Entity)
            .order_by_desc(profile_entity::Column::Rating)
            .limit(limit.min(100))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(rows
            .into_iter()
            .map(|(model, users)| dto_from_related(model, &users))
            .collect())
    }

    pub async fn list_paginated(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<CreatorProfileDto>, u64), AppError> {
        let total = profile_entity::Entity::find()
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let rows = profile_entity::Entity::find()
            .find_with_related(user_entity::Entity)
            .order_by_desc(profile_entity::Column::Rating)
            .offset((page.saturating_sub(1)) * page_size)
            .limit(page_size)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok((
            rows.into_iter()
                .map(|(model, users)| dto_from_related(model, &users))
                .collect(),
            total,
        ))
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

    async fn to_dto_with_identity(
        &self,
        model: CreatorProfileModel,
    ) -> Result<CreatorProfileDto, AppError> {
        let user = user_entity::Entity::find_by_id(model.user_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        let (nickname, avatar_url) = user
            .map(|user| (user.nickname, user.avatar_url))
            .unwrap_or_else(|| ("匿名创作者".into(), None));
        Ok(to_dto(model, nickname, avatar_url))
    }
}

fn clean(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty())
}

fn dto_from_related(model: CreatorProfileModel, users: &[user_entity::Model]) -> CreatorProfileDto {
    let (nickname, avatar_url) = users
        .first()
        .map(|user| (user.nickname.clone(), user.avatar_url.clone()))
        .unwrap_or_else(|| ("匿名创作者".into(), None));
    to_dto(model, nickname, avatar_url)
}

fn to_dto(
    model: CreatorProfileModel,
    nickname: String,
    avatar_url: Option<String>,
) -> CreatorProfileDto {
    CreatorProfileDto {
        id: model.id,
        user_id: model.user_id,
        nickname,
        avatar_url,
        introduction: model.introduction,
        bio: model.bio,
        rating: model.rating,
        certification_level: model.certification_level,
        service_areas: model.service_areas,
        style_vector_id: model.style_vector_id,
        portfolio_url: model.portfolio_url,
        total_services: model.total_services,
        total_appointments: model.total_appointments,
        avg_rating: model.avg_rating,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::entities::user as user_entity;
    use db::migrations;

    #[tokio::test]
    async fn paginated_creators_return_total_and_page_sizes() {
        let db = db::connect("sqlite::memory:").await.unwrap();
        migrations::run(&db).await.unwrap();

        for index in 0..3 {
            let now = Utc::now();
            user_entity::ActiveModel {
                username: Set(format!("creator_{index}")),
                password_hash: Set("hash".into()),
                nickname: Set(format!("Creator {index}")),
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

        let service = CreatorService::new(db);
        for user_id in 1..=3 {
            service
                .ensure_profile(
                    user_id,
                    UpsertProfileInput {
                        introduction: None,
                        bio: None,
                        service_areas: None,
                        portfolio_url: None,
                    },
                )
                .await
                .unwrap();
        }

        let (items, total) = service.list_paginated(1, 2).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(items.len(), 2);
        let (items, total) = service.list_paginated(2, 2).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(items.len(), 1);

        let json = serde_json::to_value(service.by_id(1).await.unwrap()).unwrap();
        assert!(json.get("total_income").is_none());
    }
}
