use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{
    appointment as appointment_entity, creator_profile as creator_entity,
    notification as notification_entity, user as user_entity,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDto {
    pub id: i32,
    pub user_id: i32,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub title: String,
    pub content: Option<String>,
    pub priority: String,
    pub action_url: Option<String>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct NotificationService {
    db: DatabaseConnection,
}

impl NotificationService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: i32,
        notification_type: impl Into<String>,
        title: impl Into<String>,
        content: Option<String>,
        action_url: Option<String>,
    ) -> Result<NotificationDto, AppError> {
        let model = notification_entity::ActiveModel {
            user_id: Set(user_id),
            r#type: Set(notification_type.into()),
            title: Set(title.into()),
            content: Set(content),
            priority: Set("normal".into()),
            action_url: Set(action_url),
            is_read: Set(false),
            created_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(AppError::from_anyhow)?;

        Ok(to_dto(model))
    }

    pub async fn list_for_user(
        &self,
        user_id: i32,
        limit: Option<u64>,
    ) -> Result<Vec<NotificationDto>, AppError> {
        let limit = limit.unwrap_or(20).clamp(1, 100);
        let notifications = notification_entity::Entity::find()
            .filter(notification_entity::Column::UserId.eq(user_id))
            .order_by_desc(notification_entity::Column::CreatedAt)
            .order_by_desc(notification_entity::Column::Id)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;

        Ok(notifications.into_iter().map(to_dto).collect())
    }

    pub async fn unread_count(&self, user_id: i32) -> Result<u64, AppError> {
        notification_entity::Entity::find()
            .filter(
                notification_entity::Column::UserId
                    .eq(user_id)
                    .and(notification_entity::Column::IsRead.eq(false)),
            )
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)
    }

    pub async fn mark_read(&self, id: i32, user_id: i32) -> Result<(), AppError> {
        let notification = notification_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("notification not found".into()))?;

        if notification.user_id != user_id {
            return Err(AppError::Forbidden(
                "notification does not belong to this user".into(),
            ));
        }

        if !notification.is_read {
            let mut active = notification.into_active_model();
            active.is_read = Set(true);
            active
                .update(&self.db)
                .await
                .map_err(AppError::from_anyhow)?;
        }

        Ok(())
    }

    pub async fn mark_all_read(&self, user_id: i32) -> Result<(), AppError> {
        notification_entity::Entity::update_many()
            .col_expr(
                notification_entity::Column::IsRead,
                sea_orm::sea_query::Expr::value(true),
            )
            .filter(
                notification_entity::Column::UserId
                    .eq(user_id)
                    .and(notification_entity::Column::IsRead.eq(false)),
            )
            .exec(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;

        Ok(())
    }

    pub async fn create_for_creator(
        &self,
        creator_id: i32,
        notification_type: impl Into<String>,
        title: impl Into<String>,
        content: Option<String>,
        action_url: Option<String>,
    ) -> Result<NotificationDto, AppError> {
        let creator = creator_entity::Entity::find_by_id(creator_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("creator profile not found".into()))?;

        self.create(
            creator.user_id,
            notification_type,
            title,
            content,
            action_url,
        )
        .await
    }

    pub async fn create_for_appointment_creator(
        &self,
        appointment_id: i32,
        notification_type: impl Into<String>,
        title: impl Into<String>,
        content: Option<String>,
        action_url: Option<String>,
    ) -> Result<NotificationDto, AppError> {
        let appointment = appointment_entity::Entity::find_by_id(appointment_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("appointment not found".into()))?;

        self.create_for_creator(
            appointment.creator_id,
            notification_type,
            title,
            content,
            action_url,
        )
        .await
    }

    pub async fn create_for_admins(
        &self,
        notification_type: impl Into<String>,
        title: impl Into<String>,
        content: Option<String>,
        action_url: Option<String>,
    ) -> Result<usize, AppError> {
        let notification_type = notification_type.into();
        let title = title.into();
        let admins = user_entity::Entity::find()
            .filter(
                user_entity::Column::Role
                    .eq("admin")
                    .and(user_entity::Column::Status.eq("active")),
            )
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;

        let mut created = 0;
        for admin in admins {
            self.create(
                admin.id,
                notification_type.clone(),
                title.clone(),
                content.clone(),
                action_url.clone(),
            )
            .await?;
            created += 1;
        }

        Ok(created)
    }
}

fn to_dto(model: notification_entity::Model) -> NotificationDto {
    NotificationDto {
        id: model.id,
        user_id: model.user_id,
        notification_type: model.r#type,
        title: model.title,
        content: model.content,
        priority: model.priority,
        action_url: model.action_url,
        is_read: model.is_read,
        created_at: model.created_at,
    }
}
