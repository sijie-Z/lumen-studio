use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{
    appointment as appt_entity, creator_profile as profile_entity, service as service_entity,
    user as user_entity, work as work_entity,
};
use rust_decimal::Decimal;
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait};
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
        let users = user_entity::Entity::find()
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(users
            .into_iter()
            .map(|u| AdminUserDto {
                id: u.id,
                username: u.username,
                nickname: u.nickname,
                role: u.role,
                status: u.status,
                created_at: u.created_at,
            })
            .collect())
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
        let gross_volume = appt_entity::Entity::find()
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .into_iter()
            .fold(Decimal::ZERO, |acc, item| acc + item.total_price);

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
