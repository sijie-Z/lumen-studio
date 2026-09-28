use chrono::{DateTime, Utc};
use common::{validation::max_chars, AppError};
use db::entities::{service as service_entity, service_type as type_entity, ServiceModel};
use rust_decimal::Decimal;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Select, Set,
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
        max_chars(Some(&input.title), 255, "service title")?;
        max_chars(input.description.as_deref(), 5000, "service description")?;
        let money =
            app_core::Money::new(input.price).map_err(|e| AppError::BadRequest(e.to_string()))?;
        if money.amount() <= Decimal::ZERO {
            return Err(AppError::BadRequest(
                "price must be greater than zero".into(),
            ));
        }
        if let Some(duration) = input.duration {
            if !(60..=480).contains(&duration) {
                return Err(AppError::BadRequest(
                    "duration must be between 60 and 480 minutes".into(),
                ));
            }
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
        let models = apply_service_filters(service_entity::Entity::find(), None, None, None)
            .order_by_desc(service_entity::Column::CreatedAt)
            .limit(limit.min(100))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(models.into_iter().map(to_dto).collect())
    }

    pub async fn list_active_paginated(
        &self,
        page: u64,
        page_size: u64,
        type_id: Option<i32>,
        query: Option<String>,
        location: Option<String>,
    ) -> Result<(Vec<ServiceDto>, u64), AppError> {
        let total = apply_service_filters(
            service_entity::Entity::find(),
            type_id,
            query.as_deref(),
            location.as_deref(),
        )
        .count(&self.db)
        .await
        .map_err(AppError::from_anyhow)?;

        let models = apply_service_filters(
            service_entity::Entity::find(),
            type_id,
            query.as_deref(),
            location.as_deref(),
        )
        .order_by_desc(service_entity::Column::CreatedAt)
        .offset((page.saturating_sub(1)) * page_size)
        .limit(page_size)
        .all(&self.db)
        .await
        .map_err(AppError::from_anyhow)?;

        Ok((models.into_iter().map(to_dto).collect(), total))
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
            .filter(service_entity::Column::IsActive.eq(true))
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

fn apply_service_filters(
    mut query: Select<service_entity::Entity>,
    type_id: Option<i32>,
    keyword: Option<&str>,
    location: Option<&str>,
) -> Select<service_entity::Entity> {
    query = query.filter(service_entity::Column::IsActive.eq(true));
    if let Some(type_id) = type_id {
        query = query.filter(service_entity::Column::TypeId.eq(type_id));
    }
    if let Some(keyword) = keyword.filter(|value| !value.trim().is_empty()) {
        query = query.filter(Expr::cust_with_values(
            "title LIKE ? ESCAPE '\\'",
            [format!(
                "%{}%",
                common::validation::escape_like_pattern(keyword.trim())
            )],
        ));
    }
    if let Some(location) = location.filter(|value| !value.trim().is_empty()) {
        query = query.filter(Expr::cust_with_values(
            "location LIKE ? ESCAPE '\\'",
            [format!(
                "%{}%",
                common::validation::escape_like_pattern(location.trim())
            )],
        ));
    }
    query
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

#[cfg(test)]
mod tests {
    use super::*;
    use db::entities::{creator_profile as creator_entity, user as user_entity};
    use db::migrations;
    use sea_orm::{EntityTrait, PaginatorTrait};

    #[tokio::test]
    async fn paginated_active_services_apply_type_and_text_filters() {
        let db = db::connect("sqlite::memory:").await.unwrap();
        migrations::run(&db).await.unwrap();
        let catalog = ServiceCatalog::new(db.clone());
        catalog.ensure_default_types().await.unwrap();

        for index in 0..3 {
            let now = Utc::now();
            user_entity::ActiveModel {
                username: Set(format!("service_creator_{index}")),
                password_hash: Set("hash".into()),
                nickname: Set(format!("Service Creator {index}")),
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
                user_id: Set(index + 1),
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
        }

        for (creator_id, title, location, type_name) in [
            (1, "杭州街拍", "杭州", "人像摄影"),
            (2, "上海婚礼", "上海", "婚礼纪实"),
            (3, "杭州婚礼", "杭州", "婚礼纪实"),
        ] {
            let type_id = type_entity::Entity::find()
                .filter(type_entity::Column::Name.eq(type_name))
                .one(&db)
                .await
                .unwrap()
                .unwrap()
                .id;
            catalog
                .create(
                    creator_id,
                    CreateServiceInput {
                        type_id,
                        title: title.into(),
                        description: None,
                        price: Decimal::new(100_00, 2),
                        duration: Some(120),
                        cover_image_url: None,
                        location: Some(location.into()),
                        tags: None,
                        options: None,
                    },
                )
                .await
                .unwrap();
        }

        for duration in [30, 600] {
            let result = catalog
                .create(
                    1,
                    CreateServiceInput {
                        type_id: 999,
                        title: format!("Invalid duration {duration}"),
                        description: None,
                        price: Decimal::new(100_00, 2),
                        duration: Some(duration),
                        cover_image_url: None,
                        location: None,
                        tags: None,
                        options: None,
                    },
                )
                .await;
            assert!(matches!(result, Err(AppError::BadRequest(_))));
        }

        let (items, total) = catalog
            .list_active_paginated(1, 2, None, None, None)
            .await
            .unwrap();
        assert_eq!(total, 3);
        assert_eq!(items.len(), 2);

        let wedding_type_id = type_entity::Entity::find()
            .filter(type_entity::Column::Name.eq("婚礼纪实"))
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .id;
        let (items, total) = catalog
            .list_active_paginated(1, 20, Some(wedding_type_id), Some("婚礼".into()), None)
            .await
            .unwrap();
        assert_eq!(total, 2);
        assert!(items.iter().all(|service| service.title.contains("婚礼")));

        let (items, total) = catalog
            .list_active_paginated(1, 20, None, None, Some("杭州".into()))
            .await
            .unwrap();
        assert_eq!(total, 2);
        assert!(items
            .iter()
            .all(|service| service.location.as_deref() == Some("杭州")));

        assert_eq!(service_entity::Entity::find().count(&db).await.unwrap(), 3);

        let service_id = catalog
            .create(
                1,
                CreateServiceInput {
                    type_id: wedding_type_id,
                    title: "Inactive service".into(),
                    description: None,
                    price: Decimal::new(100_00, 2),
                    duration: Some(120),
                    cover_image_url: None,
                    location: Some("杭州".into()),
                    tags: None,
                    options: None,
                },
            )
            .await
            .unwrap()
            .id;
        catalog.set_active(1, service_id, false).await.unwrap();
        assert!(matches!(
            catalog.by_id(service_id).await,
            Err(AppError::NotFound(_))
        ));
        assert!(catalog
            .list_by_creator(1)
            .await
            .unwrap()
            .iter()
            .any(|service| service.id == service_id && !service.is_active));
    }

    #[tokio::test]
    async fn service_search_escapes_like_wildcards_and_limits_title_length() {
        let db = db::connect("sqlite::memory:").await.unwrap();
        migrations::run(&db).await.unwrap();
        let catalog = ServiceCatalog::new(db.clone());
        catalog.ensure_default_types().await.unwrap();
        let now = Utc::now();

        user_entity::ActiveModel {
            username: Set("escape_service_creator".into()),
            password_hash: Set("hash".into()),
            nickname: Set("Escape Service Creator".into()),
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

        let type_id = type_entity::Entity::find()
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .id;
        for title in ["100% wedding", "ordinary service"] {
            catalog
                .create(
                    1,
                    CreateServiceInput {
                        type_id,
                        title: title.into(),
                        description: None,
                        price: Decimal::new(100_00, 2),
                        duration: Some(120),
                        cover_image_url: None,
                        location: Some("杭州".into()),
                        tags: None,
                        options: None,
                    },
                )
                .await
                .unwrap();
        }

        let (matches, total) = catalog
            .list_active_paginated(1, 20, None, Some("%".into()), None)
            .await
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(matches[0].title, "100% wedding");

        assert!(matches!(
            catalog
                .create(
                    1,
                    CreateServiceInput {
                        type_id,
                        title: "x".repeat(256),
                        description: None,
                        price: Decimal::new(100_00, 2),
                        duration: Some(120),
                        cover_image_url: None,
                        location: None,
                        tags: None,
                        options: None,
                    },
                )
                .await,
            Err(AppError::BadRequest(_))
        ));
    }
}
