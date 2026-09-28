use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::favorite as favorite_entity;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};

/// 允许收藏的目标类型。未知类型一律按非法参数拒绝。
pub const TARGET_TYPES: [&str; 3] = ["work", "service", "creator"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteToggleDto {
    pub favorited: bool,
    pub count: i64,
}

/// 当前用户对某个目标的收藏状态与该目标的总收藏数。
pub type FavoriteStatusDto = FavoriteToggleDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteDto {
    pub target_type: String,
    pub target_id: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct FavoriteService {
    db: DatabaseConnection,
}

impl FavoriteService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn normalize_target_type(target_type: &str) -> Result<String, AppError> {
        let value = target_type.trim().to_ascii_lowercase();
        if TARGET_TYPES.contains(&value.as_str()) {
            Ok(value)
        } else {
            Err(AppError::BadRequest(
                "target_type must be one of work, service, creator".into(),
            ))
        }
    }

    /// 切换收藏状态：已收藏则取消，未收藏则新增。返回最新状态和总收藏数。
    pub async fn toggle(
        &self,
        user_id: i32,
        target_type: &str,
        target_id: i32,
    ) -> Result<FavoriteToggleDto, AppError> {
        let target_type = Self::normalize_target_type(target_type)?;
        let existing = favorite_entity::Entity::find()
            .filter(
                favorite_entity::Column::UserId
                    .eq(user_id)
                    .and(favorite_entity::Column::TargetType.eq(target_type.clone()))
                    .and(favorite_entity::Column::TargetId.eq(target_id)),
            )
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;

        let favorited = match existing {
            Some(model) => {
                favorite_entity::Entity::delete_by_id(model.id)
                    .exec(&self.db)
                    .await
                    .map_err(AppError::from_anyhow)?;
                false
            }
            None => {
                let insert = favorite_entity::ActiveModel {
                    user_id: Set(user_id),
                    target_type: Set(target_type.clone()),
                    target_id: Set(target_id),
                    created_at: Set(Utc::now()),
                    ..Default::default()
                }
                .insert(&self.db)
                .await;

                match insert {
                    Ok(_) => true,
                    // 并发切换时唯一约束可能先被另一个请求写入，此时视为已收藏。
                    Err(err) if err.to_string().to_lowercase().contains("unique") => true,
                    Err(err) => return Err(AppError::from_anyhow(err)),
                }
            }
        };

        let count = self.count(&target_type, target_id).await?;
        Ok(FavoriteToggleDto { favorited, count })
    }

    /// 当前用户是否收藏了某个目标。
    pub async fn is_favorited(
        &self,
        user_id: i32,
        target_type: &str,
        target_id: i32,
    ) -> Result<bool, AppError> {
        let target_type = Self::normalize_target_type(target_type)?;
        let count = favorite_entity::Entity::find()
            .filter(
                favorite_entity::Column::UserId
                    .eq(user_id)
                    .and(favorite_entity::Column::TargetType.eq(target_type))
                    .and(favorite_entity::Column::TargetId.eq(target_id)),
            )
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(count > 0)
    }

    /// 某个目标被收藏的总次数。
    pub async fn count(&self, target_type: &str, target_id: i32) -> Result<i64, AppError> {
        let target_type = Self::normalize_target_type(target_type)?;
        let count = favorite_entity::Entity::find()
            .filter(
                favorite_entity::Column::TargetType
                    .eq(target_type)
                    .and(favorite_entity::Column::TargetId.eq(target_id)),
            )
            .count(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        Ok(count as i64)
    }

    /// 状态查询：当前用户是否已收藏 + 该目标总收藏数。
    pub async fn status(
        &self,
        user_id: i32,
        target_type: &str,
        target_id: i32,
    ) -> Result<FavoriteStatusDto, AppError> {
        let target_type = Self::normalize_target_type(target_type)?;
        let favorited = self.is_favorited(user_id, &target_type, target_id).await?;
        let count = self.count(&target_type, target_id).await?;
        Ok(FavoriteStatusDto { favorited, count })
    }

    /// 当前用户的收藏列表，按收藏时间倒序。可选按目标类型过滤。
    pub async fn list(
        &self,
        user_id: i32,
        target_type: Option<&str>,
    ) -> Result<Vec<FavoriteDto>, AppError> {
        let filter_type = match target_type {
            Some(value) if !value.trim().is_empty() => Some(Self::normalize_target_type(value)?),
            _ => None,
        };

        let mut query = favorite_entity::Entity::find()
            .filter(favorite_entity::Column::UserId.eq(user_id))
            .order_by_desc(favorite_entity::Column::CreatedAt)
            .order_by_desc(favorite_entity::Column::Id);

        if let Some(target_type) = filter_type {
            query = query.filter(favorite_entity::Column::TargetType.eq(target_type));
        }

        let rows = query.all(&self.db).await.map_err(AppError::from_anyhow)?;
        Ok(rows.into_iter().map(to_dto).collect())
    }
}

fn to_dto(model: favorite_entity::Model) -> FavoriteDto {
    FavoriteDto {
        target_type: model.target_type,
        target_id: model.target_id,
        created_at: model.created_at,
    }
}
