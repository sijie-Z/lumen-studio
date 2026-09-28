use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use common::AppError;
use db::entities::{
    creator_profile as creator_entity, favorite as favorite_entity, service as service_entity,
    user as user_entity, work as work_entity,
};
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
    pub id: i32,
    pub target_type: String,
    pub target_id: i32,
    pub title: String,
    pub cover_image_url: Option<String>,
    pub subtitle: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct FavoritePresentation {
    title: String,
    cover_image_url: Option<String>,
    subtitle: Option<String>,
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

    async fn target_exists(&self, target_type: &str, target_id: i32) -> Result<bool, AppError> {
        let exists = match target_type {
            "work" => work_entity::Entity::find_by_id(target_id)
                .one(&self.db)
                .await
                .map_err(AppError::from_anyhow)?
                .is_some(),
            "service" => service_entity::Entity::find_by_id(target_id)
                .one(&self.db)
                .await
                .map_err(AppError::from_anyhow)?
                .is_some(),
            "creator" => creator_entity::Entity::find_by_id(target_id)
                .one(&self.db)
                .await
                .map_err(AppError::from_anyhow)?
                .is_some(),
            _ => unreachable!("target type was normalized before this call"),
        };

        Ok(exists)
    }

    /// 切换收藏状态：已收藏则取消，未收藏则新增。返回最新状态和总收藏数。
    pub async fn toggle(
        &self,
        user_id: i32,
        target_type: &str,
        target_id: i32,
    ) -> Result<FavoriteToggleDto, AppError> {
        let target_type = Self::normalize_target_type(target_type)?;
        if !self.target_exists(&target_type, target_id).await? {
            return Err(AppError::NotFound(format!(
                "favorite target {target_type} #{target_id} does not exist"
            )));
        }

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
        let mut presentations = HashMap::new();

        let work_ids = ids_for(&rows, "work");
        if !work_ids.is_empty() {
            presentations.extend(self.load_work_presentations(work_ids).await?);
        }

        let service_ids = ids_for(&rows, "service");
        if !service_ids.is_empty() {
            presentations.extend(self.load_service_presentations(service_ids).await?);
        }

        let creator_ids = ids_for(&rows, "creator");
        if !creator_ids.is_empty() {
            presentations.extend(self.load_creator_presentations(creator_ids).await?);
        }

        Ok(rows
            .into_iter()
            .map(|model| {
                let key = (model.target_type.clone(), model.target_id);
                let presentation = presentations
                    .remove(&key)
                    .unwrap_or_else(|| fallback_presentation(&model.target_type, model.target_id));

                FavoriteDto {
                    id: model.id,
                    target_type: model.target_type,
                    target_id: model.target_id,
                    title: presentation.title,
                    cover_image_url: presentation.cover_image_url,
                    subtitle: presentation.subtitle,
                    created_at: model.created_at,
                }
            })
            .collect())
    }

    async fn load_work_presentations(
        &self,
        ids: Vec<i32>,
    ) -> Result<HashMap<(String, i32), FavoritePresentation>, AppError> {
        let works = work_entity::Entity::find()
            .filter(work_entity::Column::Id.is_in(ids))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;

        let user_ids = works
            .iter()
            .map(|work| work.user_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let users = user_entity::Entity::find()
            .filter(user_entity::Column::Id.is_in(user_ids))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .into_iter()
            .map(|user| (user.id, user))
            .collect::<HashMap<_, _>>();

        Ok(works
            .into_iter()
            .map(|work| {
                let title =
                    clean_string(work.title).unwrap_or_else(|| format!("作品 #{}", work.id));
                let subtitle = users
                    .get(&work.user_id)
                    .map(display_user_name)
                    .filter(|name| !name.trim().is_empty());
                (
                    ("work".into(), work.id),
                    FavoritePresentation {
                        title,
                        cover_image_url: clean_string(Some(work.image_url)),
                        subtitle,
                    },
                )
            })
            .collect())
    }

    async fn load_service_presentations(
        &self,
        ids: Vec<i32>,
    ) -> Result<HashMap<(String, i32), FavoritePresentation>, AppError> {
        let services = service_entity::Entity::find()
            .filter(service_entity::Column::Id.is_in(ids))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;

        Ok(services
            .into_iter()
            .map(|service| {
                let subtitle = clean_string(service.location)
                    .or_else(|| Some(format!("¥{}", service.price.normalize())));
                (
                    ("service".into(), service.id),
                    FavoritePresentation {
                        title: service.title,
                        cover_image_url: clean_string(service.cover_image_url),
                        subtitle,
                    },
                )
            })
            .collect())
    }

    async fn load_creator_presentations(
        &self,
        ids: Vec<i32>,
    ) -> Result<HashMap<(String, i32), FavoritePresentation>, AppError> {
        let creators = creator_entity::Entity::find()
            .filter(creator_entity::Column::Id.is_in(ids))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;

        let user_ids = creators
            .iter()
            .map(|creator| creator.user_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let users = user_entity::Entity::find()
            .filter(user_entity::Column::Id.is_in(user_ids))
            .all(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .into_iter()
            .map(|user| (user.id, user))
            .collect::<HashMap<_, _>>();

        Ok(creators
            .into_iter()
            .map(|creator| {
                let user = users.get(&creator.user_id);
                let title = user
                    .map(display_user_name)
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or_else(|| format!("创作者 #{}", creator.id));
                let subtitle = clean_string(creator.introduction)
                    .or_else(|| clean_string(creator.bio))
                    .or_else(|| user.map(|user| format!("@{}", user.username)));
                (
                    ("creator".into(), creator.id),
                    FavoritePresentation {
                        title,
                        cover_image_url: user
                            .and_then(|user| clean_string(user.avatar_url.clone())),
                        subtitle,
                    },
                )
            })
            .collect())
    }
}

fn ids_for(rows: &[favorite_entity::Model], target_type: &str) -> Vec<i32> {
    rows.iter()
        .filter(|row| row.target_type == target_type)
        .map(|row| row.target_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

fn clean_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn display_user_name(user: &user_entity::Model) -> String {
    let nickname = user.nickname.trim();
    if !nickname.is_empty() {
        nickname.to_string()
    } else {
        user.username.clone()
    }
}

fn fallback_presentation(target_type: &str, target_id: i32) -> FavoritePresentation {
    let label = match target_type {
        "work" => "作品",
        "service" => "服务",
        "creator" => "创作者",
        _ => "收藏",
    };
    FavoritePresentation {
        title: format!("{label} #{target_id}"),
        cover_image_url: None,
        subtitle: Some("目标已不可用".into()),
    }
}
