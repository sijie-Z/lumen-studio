use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{user as user_entity, work as work_entity, WorkModel};
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect, Set,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WorkDto {
    pub id: i32,
    pub user_id: i32,
    pub image_url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub creator_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct WorkService {
    db: DatabaseConnection,
}

impl WorkService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: i32,
        image_url: String,
        title: Option<String>,
        description: Option<String>,
        category: Option<String>,
    ) -> Result<WorkDto, AppError> {
        if image_url.trim().is_empty() {
            return Err(AppError::BadRequest("image_url is required".into()));
        }

        let model = work_entity::ActiveModel {
            user_id: Set(user_id),
            image_url: Set(image_url),
            title: Set(title.filter(|value| !value.trim().is_empty())),
            description: Set(description.filter(|value| !value.trim().is_empty())),
            category: Set(category.filter(|value| !value.trim().is_empty())),
            created_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(AppError::from_anyhow)?;

        let creator_name = user_entity::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .map(|user| user.nickname)
            .unwrap_or_else(|| "匿名创作者".into());
        Ok(to_dto(model, creator_name))
    }

    pub async fn list(&self, limit: u64) -> Result<Vec<WorkDto>, AppError> {
        let rows = work_entity::Entity::find()
            .find_with_related(user_entity::Entity)
            .order_by_desc(work_entity::Column::CreatedAt)
            .limit(limit.min(100))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(rows
            .into_iter()
            .map(|(work, users)| {
                let creator_name = users
                    .first()
                    .map(|user| user.nickname.clone())
                    .unwrap_or_else(|| "匿名创作者".into());
                to_dto(work, creator_name)
            })
            .collect())
    }

    pub async fn count(&self) -> Result<u64, AppError> {
        work_entity::Entity::find()
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)
    }
}

fn to_dto(model: WorkModel, creator_name: String) -> WorkDto {
    WorkDto {
        id: model.id,
        user_id: model.user_id,
        image_url: model.image_url,
        title: model.title,
        description: model.description,
        category: model.category,
        creator_name,
        created_at: model.created_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::entities::user as user_entity;
    use db::migrations;

    #[tokio::test]
    async fn create_list_and_count_works() {
        let db = db::connect("sqlite::memory:").await.unwrap();
        migrations::run(&db).await.unwrap();

        let now = Utc::now();
        user_entity::ActiveModel {
            username: Set("artist".into()),
            password_hash: Set("hash".into()),
            nickname: Set("Artist".into()),
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

        let service = WorkService::new(db);
        let created = service
            .create(1, "/uploads/one.jpg".into(), Some("One".into()), None, None)
            .await
            .unwrap();
        assert_eq!(created.image_url, "/uploads/one.jpg");
        assert_eq!(service.count().await.unwrap(), 1);
        assert_eq!(service.list(20).await.unwrap().len(), 1);
    }
}
