use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{appointment as appt_entity, service as service_entity, AppointmentModel};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct AppointmentDto {
    pub id: i32,
    pub user_id: i32,
    pub creator_id: i32,
    pub service_id: i32,
    pub appointment_date: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub status: String,
    pub total_price: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAppointmentInput {
    pub service_id: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone)]
pub struct AppointmentService {
    db: DatabaseConnection,
}

impl AppointmentService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: i32,
        input: CreateAppointmentInput,
    ) -> Result<AppointmentDto, AppError> {
        let service = service_entity::Entity::find_by_id(input.service_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("service not found".into()))?;
        if !service.is_active {
            return Err(AppError::BadRequest("service is not available".into()));
        }

        let slot = app_core::TimeSlot::new(input.start_time, input.end_time)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let conflict = appt_entity::Entity::find()
            .filter(
                appt_entity::Column::CreatorId
                    .eq(service.creator_id)
                    .and(appt_entity::Column::StartTime.lt(input.end_time))
                    .and(appt_entity::Column::EndTime.gt(input.start_time))
                    .and(appt_entity::Column::Status.is_not_in(["cancelled", "refunded"])),
            )
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        if conflict.is_some() {
            return Err(AppError::Conflict("time slot is already booked".into()));
        }

        let now = Utc::now();
        let model = appt_entity::ActiveModel {
            user_id: Set(user_id),
            creator_id: Set(service.creator_id),
            service_id: Set(service.id),
            appointment_date: Set(input.start_time.format("%Y-%m-%d").to_string()),
            start_time: Set(slot.start),
            end_time: Set(slot.end),
            location: Set(input.location.filter(|v| !v.trim().is_empty())),
            status: Set("pending".into()),
            total_price: Set(service.price),
            notes: Set(input.notes.filter(|v| !v.trim().is_empty())),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(AppError::from_anyhow)?;

        self.bump_counts(service.id, service.creator_id).await?;

        Ok(to_dto(model))
    }

    pub async fn list_for_user(&self, user_id: i32) -> Result<Vec<AppointmentDto>, AppError> {
        let models = appt_entity::Entity::find()
            .filter(appt_entity::Column::UserId.eq(user_id))
            .order_by_desc(appt_entity::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(models.into_iter().map(to_dto).collect())
    }

    pub async fn list_for_creator(&self, creator_id: i32) -> Result<Vec<AppointmentDto>, AppError> {
        let models = appt_entity::Entity::find()
            .filter(appt_entity::Column::CreatorId.eq(creator_id))
            .order_by_desc(appt_entity::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(models.into_iter().map(to_dto).collect())
    }

    pub async fn transition(
        &self,
        actor_user_id: i32,
        actor_creator_id: Option<i32>,
        appointment_id: i32,
        target: String,
    ) -> Result<AppointmentDto, AppError> {
        let model = appt_entity::Entity::find_by_id(appointment_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("appointment not found".into()))?;

        let is_client = model.user_id == actor_user_id;
        let is_creator = actor_creator_id == Some(model.creator_id);
        if !is_client && !is_creator {
            return Err(AppError::Forbidden(
                "not allowed to modify this appointment".into(),
            ));
        }

        let current = status_from_str(&model.status)?;
        let next = status_from_str(&target)?;
        if !current.can_transition_to(&next) {
            return Err(AppError::BadRequest(format!(
                "invalid transition: {current:?} -> {target}"
            )));
        }

        // Clients may only cancel; creators drive the rest of the flow.
        let client_only_targets = ["cancelled"];
        if is_client && !is_creator && !client_only_targets.contains(&target.as_str()) {
            return Err(AppError::Forbidden(
                "only the creator can perform this action".into(),
            ));
        }

        let mut active = model.into_active_model();
        active.status = Set(target);
        active.updated_at = Set(Utc::now());
        let updated = active
            .update(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(to_dto(updated))
    }

    async fn bump_counts(&self, service_id: i32, creator_id: i32) -> Result<(), AppError> {
        if let Some(service) = service_entity::Entity::find_by_id(service_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
        {
            let count = service.appointments_count;
            let mut active = service.into_active_model();
            active.appointments_count = Set(count + 1);
            active
                .update(&self.db)
                .await
                .map_err(AppError::from_anyhow)?;
        }
        if let Some(profile) = db::entities::creator_profile::Entity::find_by_id(creator_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
        {
            let appointments = profile.total_appointments;
            let mut active = profile.into_active_model();
            active.total_appointments = Set(appointments + 1);
            active
                .update(&self.db)
                .await
                .map_err(AppError::from_anyhow)?;
        }
        Ok(())
    }
}

fn status_from_str(value: &str) -> Result<app_core::AppointmentStatus, AppError> {
    match value {
        "pending" => Ok(app_core::AppointmentStatus::Pending),
        "confirmed" => Ok(app_core::AppointmentStatus::Confirmed),
        "ongoing" => Ok(app_core::AppointmentStatus::Ongoing),
        "completed" => Ok(app_core::AppointmentStatus::Completed),
        "cancelled" => Ok(app_core::AppointmentStatus::Cancelled),
        "refunded" => Ok(app_core::AppointmentStatus::Refunded),
        other => Err(AppError::BadRequest(format!("unknown status: {other}"))),
    }
}

fn to_dto(model: AppointmentModel) -> AppointmentDto {
    AppointmentDto {
        id: model.id,
        user_id: model.user_id,
        creator_id: model.creator_id,
        service_id: model.service_id,
        appointment_date: model.appointment_date,
        start_time: model.start_time,
        end_time: model.end_time,
        location: model.location,
        status: model.status,
        total_price: model.total_price,
        notes: model.notes,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creator_service::{CreatorService, UpsertProfileInput};
    use crate::service_catalog::{CreateServiceInput, ServiceCatalog};
    use chrono::Duration;
    use db::entities::user as user_entity;
    use db::migrations;

    async fn insert_user(db: &DatabaseConnection, id: i32, username: &str) {
        let now = Utc::now();
        user_entity::ActiveModel {
            id: Set(id),
            username: Set(username.to_string()),
            password_hash: Set("hash".into()),
            nickname: Set(username.to_string()),
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

    #[tokio::test]
    async fn create_conflict_and_state_machine() {
        let db = db::connect("sqlite::memory:").await.unwrap();
        migrations::run(&db).await.unwrap();
        insert_user(&db, 1, "creator").await;
        insert_user(&db, 2, "client").await;

        let creators = CreatorService::new(db.clone());
        let creator = creators
            .ensure_profile(
                1,
                UpsertProfileInput {
                    introduction: Some("Portrait photographer".into()),
                    bio: None,
                    service_areas: None,
                    portfolio_url: None,
                },
            )
            .await
            .unwrap();

        let catalog = ServiceCatalog::new(db.clone());
        catalog.ensure_default_types().await.unwrap();
        let type_id = catalog.list_types().await.unwrap()[0].id;
        let service = catalog
            .create(
                creator.id,
                CreateServiceInput {
                    type_id,
                    title: "Outdoor portrait".into(),
                    description: None,
                    price: Decimal::new(3999_00, 2),
                    duration: Some(120),
                    cover_image_url: None,
                    location: Some("Shanghai".into()),
                    tags: None,
                    options: None,
                },
            )
            .await
            .unwrap();

        let appointments = AppointmentService::new(db.clone());
        let start = Utc::now() + Duration::days(1);
        let end = start + Duration::minutes(120);

        let created = appointments
            .create(
                2,
                CreateAppointmentInput {
                    service_id: service.id,
                    start_time: start,
                    end_time: end,
                    location: None,
                    notes: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(created.status, "pending");
        let profile = db::entities::creator_profile::Entity::find_by_id(creator.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(profile.total_appointments, 1);
        assert_eq!(profile.total_income, Decimal::ZERO);

        let conflict = appointments
            .create(
                2,
                CreateAppointmentInput {
                    service_id: service.id,
                    start_time: start + Duration::minutes(30),
                    end_time: end + Duration::minutes(30),
                    location: None,
                    notes: None,
                },
            )
            .await;
        assert!(matches!(conflict, Err(AppError::Conflict(_))));

        let forbidden = appointments
            .transition(2, None, created.id, "confirmed".into())
            .await;
        assert!(matches!(forbidden, Err(AppError::Forbidden(_))));

        let confirmed = appointments
            .transition(1, Some(creator.id), created.id, "confirmed".into())
            .await
            .unwrap();
        assert_eq!(confirmed.status, "confirmed");

        let completed = appointments
            .transition(1, Some(creator.id), created.id, "ongoing".into())
            .await
            .unwrap();
        assert_eq!(completed.status, "ongoing");
    }
}
