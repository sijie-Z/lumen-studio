use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{service as service_entity, service_type as type_entity, ServiceModel};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use serde::Serialize;

const DEFAULT_SERVICE_TYPES: [&str; 6] = [
    "人像摄影",
    "婚礼纪实",
    "商业拍摄",
    "短片影像",
    "化妆造型",
    "设计",
];

#[derive(Debug, Clone, Serialize)]
pub struct ServiceDto {
    pub id: i32,
    pub creator_id: i32,
    pub type_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub duration: Option<i32>,
    pub cover_image_url: Option<String>,
    pub location: Option<String>,
    pub tags: Option<String>,
    pub options: Option<serde_json::Value>,
    pub is_active: bool,
    pub is_featured: bool,
    pub appointments_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceTypeDto {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateServiceInput {
    pub type_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub duration: Option<i32>,
    pub cover_image_url: Option<String>,
    pub location: Option<String>,
    pub tags: Option<String>,
    pub options: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct ServiceCatalog {
    db: DatabaseConnection,
}

impl ServiceCatalog {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn ensure_default_types(&self) -> Result<(), AppError> {
        for name in DEFAULT_SERVICE_TYPES {
            let exists = type_entity::Entity::find()
                .filter(type_entity::Column::Name.eq(name))
                .one(&self.db)
                .await
                .map_err(AppError::from_anyhow)?
                .is_some();
            if !exists {
                type_entity::ActiveModel {
                    name: Set(name.to_string()),
                    description: Set(None),
                    created_at: Set(Utc::now()),
                    ..Default::default()
                }
                .insert(&self.db)
                .await
                .map_err(AppError::from_anyhow)?;
            }
        }
        Ok(())
    }

    pub async fn list_types(&self) -> Result<Vec<ServiceTypeDto>, AppError> {
        let models = type_entity::Entity::find()
            .order_by_asc(type_entity::Column::Id)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(models
            .into_iter()
            .map(|m| ServiceTypeDto {
                id: m.id,
                name: m.name,
                description: m.description,
            })
            .collect())
    }

    pub async fn create(
        &self,
        creator_id: i32,
        input: CreateServiceInput,
    ) -> Result<ServiceDto, AppError> {
        if input.title.trim().is_empty() {
            return Err(AppError::BadRequest("title is required".into()));
        }
        let money = app_core::Money::new(input.price)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        if money.amount() <= Decimal::ZERO {
            return Err(AppError::BadRequest("price must be greater than zero".into()));
        }

        let service_type = type_entity::Entity::find_by_id(input.type_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::BadRequest("service type does not exist".into()))?;

        let now = Utc::now();
        let model = service_entity::ActiveModel {
            creator_id: Set(creator_id),
            type_id: Set(service_type.id),
            title: Set(input.title.trim().to_string()),
            description: Set(clean(input.description)),
            price: Set(money.amount()),
            duration: Set(input.duration.filter(|d| *d > 0)),
            cover_image_url: Set(clean(input.cover_image_url)),
            location: Set(clean(input.location)),
            tags: Set(clean(input.tags)),
            options: Set(input.options),
            is_active: Set(true),
            is_featured: Set(false),
            appointments_count: Set(0),
            style_vector_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(AppError::from_anyhow)?;

        Ok(to_dto(model))
    }

    pub async fn list_active(&self, limit: u64) -> Result<Vec<ServiceDto>, AppError> {
        let models = service_entity::Entity::find()
            .filter(service_entity::Column::IsActive.eq(true))
            .order_by_desc(service_entity::Column::CreatedAt)
            .limit(limit.min(100))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(models.into_iter().map(to_dto).collect())
    }

    pub async fn list_by_creator(&self, creator_id: i32) -> Result<Vec<ServiceDto>, AppError> {
        let models = service_entity::Entity::find()
            .filter(service_entity::Column::CreatorId.eq(creator_id))
            .order_by_desc(service_entity::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(models.into_iter().map(to_dto).collect())
    }

    pub async fn by_id(&self, id: i32) -> Result<ServiceDto, AppError> {
        let model = service_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("service not found".into()))?;
        Ok(to_dto(model))
    }

    pub async fn set_active(
        &self,
        creator_id: i32,
        service_id: i32,
        is_active: bool,
    ) -> Result<ServiceDto, AppError> {
        let model = service_entity::Entity::find_by_id(service_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("service not found".into()))?;
        if model.creator_id != creator_id {
            return Err(AppError::Forbidden("not your service".into()));
        }
        let mut active = model.into_active_model();
        active.is_active = Set(is_active);
        active.updated_at = Set(Utc::now());
        let updated = active
            .update(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(to_dto(updated))
    }

    pub async fn increment_appointments(&self, service_id: i32) -> Result<(), AppError> {
        let model = service_entity::Entity::find_by_id(service_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("service not found".into()))?;
        let count = model.appointments_count;
        let mut active = model.into_active_model();
        active.appointments_count = Set(count + 1);
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

fn to_dto(model: ServiceModel) -> ServiceDto {
    ServiceDto {
        id: model.id,
        creator_id: model.creator_id,
        type_id: model.type_id,
        title: model.title,
        description: model.description,
        price: model.price,
        duration: model.duration,
        cover_image_url: model.cover_image_url,
        location: model.location,
        tags: model.tags,
        options: model.options,
        is_active: model.is_active,
        is_featured: model.is_featured,
        appointments_count: model.appointments_count,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}
