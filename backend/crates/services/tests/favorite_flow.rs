use chrono::Utc;
use common::AppError;
use db::entities::{favorite as favorite_entity, user as user_entity};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use services::favorite_service::FavoriteService;

async fn insert_user(db: &DatabaseConnection, id: i32) {
    let now = Utc::now();
    user_entity::ActiveModel {
        id: Set(id),
        username: Set(format!("fav-user-{id}")),
        password_hash: Set("hash".into()),
        nickname: Set(format!("Fav User {id}")),
        status: Set("active".into()),
        role: Set("user".into()),
        verification_status: Set("verified".into()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();
}

async fn rows_for(db: &DatabaseConnection, user_id: i32, target_type: &str, target_id: i32) -> u64 {
    favorite_entity::Entity::find()
        .filter(
            favorite_entity::Column::UserId
                .eq(user_id)
                .and(favorite_entity::Column::TargetType.eq(target_type))
                .and(favorite_entity::Column::TargetId.eq(target_id)),
        )
        .count(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn toggle_twice_returns_to_unfavorited() {
    let db = db::connect("sqlite::memory:").await.unwrap();
    db::migrations::run(&db).await.unwrap();
    insert_user(&db, 1).await;

    let service = FavoriteService::new(db.clone());
    let first = service.toggle(1, "work", 10).await.unwrap();
    assert!(first.favorited);
    assert_eq!(first.count, 1);

    let second = service.toggle(1, "work", 10).await.unwrap();
    assert!(!second.favorited);
    assert_eq!(second.count, 0);

    assert!(!service.is_favorited(1, "work", 10).await.unwrap());
}

#[tokio::test]
async fn repeated_favorite_never_creates_duplicate_rows() {
    let db = db::connect("sqlite::memory:").await.unwrap();
    db::migrations::run(&db).await.unwrap();
    insert_user(&db, 1).await;

    let service = FavoriteService::new(db.clone());
    service.toggle(1, "service", 7).await.unwrap();
    assert_eq!(rows_for(&db, 1, "service", 7).await, 1);

    // 收藏 -> 取消 -> 再次收藏，始终只保留一行。
    service.toggle(1, "service", 7).await.unwrap();
    service.toggle(1, "service", 7).await.unwrap();
    assert_eq!(rows_for(&db, 1, "service", 7).await, 1);
}

#[tokio::test]
async fn count_reflects_distinct_users() {
    let db = db::connect("sqlite::memory:").await.unwrap();
    db::migrations::run(&db).await.unwrap();
    insert_user(&db, 1).await;
    insert_user(&db, 2).await;

    let service = FavoriteService::new(db.clone());
    let first = service.toggle(1, "creator", 3).await.unwrap();
    assert_eq!(first.count, 1);

    let second = service.toggle(2, "creator", 3).await.unwrap();
    assert!(second.favorited);
    assert_eq!(second.count, 2);

    let removed = service.toggle(1, "creator", 3).await.unwrap();
    assert!(!removed.favorited);
    assert_eq!(removed.count, 1);
}

#[tokio::test]
async fn invalid_target_type_is_rejected() {
    let db = db::connect("sqlite::memory:").await.unwrap();
    db::migrations::run(&db).await.unwrap();
    insert_user(&db, 1).await;

    let service = FavoriteService::new(db.clone());
    assert!(matches!(
        service.toggle(1, "album", 1).await.unwrap_err(),
        AppError::BadRequest(_)
    ));
    assert!(matches!(
        service.list(1, Some("album")).await.unwrap_err(),
        AppError::BadRequest(_)
    ));
    assert!(matches!(
        service.status(1, "", 1).await.unwrap_err(),
        AppError::BadRequest(_)
    ));
}

#[tokio::test]
async fn list_filters_by_target_type() {
    let db = db::connect("sqlite::memory:").await.unwrap();
    db::migrations::run(&db).await.unwrap();
    insert_user(&db, 1).await;

    let service = FavoriteService::new(db.clone());
    service.toggle(1, "work", 1).await.unwrap();
    service.toggle(1, "creator", 3).await.unwrap();

    let all = service.list(1, None).await.unwrap();
    assert_eq!(all.len(), 2);

    let creators = service.list(1, Some("creator")).await.unwrap();
    assert_eq!(creators.len(), 1);
    assert_eq!(creators[0].target_type, "creator");
    assert_eq!(creators[0].target_id, 3);

    let empty = service.list(1, Some("service")).await.unwrap();
    assert!(empty.is_empty());
}
