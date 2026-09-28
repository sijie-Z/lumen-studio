use chrono::Utc;
use common::AppError;
use db::entities::{
    creator_profile as creator_entity, favorite as favorite_entity, service as service_entity,
    service_type as service_type_entity, user as user_entity, work as work_entity,
};
use rust_decimal::Decimal;
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
        avatar_url: Set(Some(format!("https://example.com/avatar-{id}.jpg"))),
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

async fn insert_targets(db: &DatabaseConnection) {
    let now = Utc::now();
    creator_entity::ActiveModel {
        id: Set(3),
        user_id: Set(1),
        introduction: Set(Some("Fav Creator Intro".into())),
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
    .insert(db)
    .await
    .unwrap();

    work_entity::ActiveModel {
        id: Set(10),
        user_id: Set(1),
        image_url: Set("https://example.com/work-10.jpg".into()),
        title: Set(Some("Fav Work".into())),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    service_type_entity::ActiveModel {
        id: Set(1),
        name: Set("portrait".into()),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    service_entity::ActiveModel {
        id: Set(7),
        creator_id: Set(3),
        type_id: Set(1),
        title: Set("Fav Service".into()),
        description: Set(Some("A service used by favorite tests".into())),
        price: Set(Decimal::new(39900, 2)),
        duration: Set(Some(120)),
        cover_image_url: Set(Some("https://example.com/service-7.jpg".into())),
        location: Set(Some("Shanghai".into())),
        is_active: Set(true),
        is_featured: Set(false),
        appointments_count: Set(0),
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
    insert_targets(&db).await;

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
    insert_targets(&db).await;

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
    insert_targets(&db).await;

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
    insert_targets(&db).await;

    let service = FavoriteService::new(db.clone());
    service.toggle(1, "work", 10).await.unwrap();
    service.toggle(1, "service", 7).await.unwrap();
    service.toggle(1, "creator", 3).await.unwrap();

    let all = service.list(1, None).await.unwrap();
    assert_eq!(all.len(), 3);

    let work = all.iter().find(|item| item.target_type == "work").unwrap();
    assert_eq!(work.title, "Fav Work");
    assert_eq!(
        work.cover_image_url.as_deref(),
        Some("https://example.com/work-10.jpg")
    );
    assert_eq!(work.subtitle.as_deref(), Some("Fav User 1"));

    let service_item = all
        .iter()
        .find(|item| item.target_type == "service")
        .unwrap();
    assert_eq!(service_item.title, "Fav Service");
    assert_eq!(service_item.subtitle.as_deref(), Some("Shanghai"));

    let creators = service.list(1, Some("creator")).await.unwrap();
    assert_eq!(creators.len(), 1);
    assert_eq!(creators[0].target_type, "creator");
    assert_eq!(creators[0].target_id, 3);
    assert_eq!(creators[0].title, "Fav User 1");
    assert_eq!(
        creators[0].cover_image_url.as_deref(),
        Some("https://example.com/avatar-1.jpg")
    );

    let services = service.list(1, Some("service")).await.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].target_id, 7);
}

#[tokio::test]
async fn toggle_nonexistent_target_returns_not_found() {
    let db = db::connect("sqlite::memory:").await.unwrap();
    db::migrations::run(&db).await.unwrap();
    insert_user(&db, 1).await;
    insert_targets(&db).await;

    let service = FavoriteService::new(db.clone());
    for (target_type, target_id) in [("work", 999), ("service", 999), ("creator", 999)] {
        assert!(matches!(
            service.toggle(1, target_type, target_id).await.unwrap_err(),
            AppError::NotFound(_)
        ));
    }
}
