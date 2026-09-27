use chrono::Utc;
use common::AppError;
use db::entities::{creator_profile as creator_entity, user as user_entity};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde_json::json;
use services::withdrawal_service::WithdrawalService;

async fn insert_user(db: &DatabaseConnection, id: i32, balance: Decimal) {
    let now = Utc::now();
    user_entity::ActiveModel {
        id: Set(id),
        username: Set(format!("creator-{id}")),
        password_hash: Set("hash".into()),
        nickname: Set(format!("Creator {id}")),
        balance: Set(balance),
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

async fn insert_creator(db: &DatabaseConnection, id: i32, user_id: i32) {
    let now = Utc::now();
    creator_entity::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
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
}

#[tokio::test]
async fn withdrawal_apply_review_and_refund_flow() {
    let db = db::connect("sqlite::memory:").await.unwrap();
    db::migrations::run(&db).await.unwrap();
    insert_user(&db, 1, Decimal::new(1000_00, 2)).await;
    insert_creator(&db, 1, 1).await;

    let service = WithdrawalService::new(db.clone());
    let first = service
        .apply(
            1,
            Decimal::new(200_00, 2),
            Some(json!({ "account": "alipay:creator@example.com" })),
        )
        .await
        .unwrap();
    assert_eq!(first.status, "pending");
    assert_eq!(first.actual_amount, Some(Decimal::new(200_00, 2)));

    let creator_user = user_entity::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(creator_user.balance, Decimal::new(800_00, 2));

    let rejected = service
        .review(first.id, 1, false, Some("账户信息不完整".into()))
        .await
        .unwrap();
    assert_eq!(rejected.status, "rejected");

    let creator_user = user_entity::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(creator_user.balance, Decimal::new(1000_00, 2));

    let second = service
        .apply(1, Decimal::new(300_00, 2), None)
        .await
        .unwrap();
    let completed = service
        .review(second.id, 1, true, Some("已打款".into()))
        .await
        .unwrap();
    assert_eq!(completed.status, "completed");
    assert!(completed.completed_at.is_some());

    let repeated_review = service.review(second.id, 1, true, None).await;
    assert!(matches!(repeated_review, Err(AppError::Conflict(_))));

    let creator_user = user_entity::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(creator_user.balance, Decimal::new(700_00, 2));
    assert_eq!(service.list_for_creator(1).await.unwrap().len(), 2);
    assert_eq!(service.list_all().await.unwrap().len(), 2);

    let missing_creator = service.apply(999, Decimal::new(1, 0), None).await;
    assert!(matches!(missing_creator, Err(AppError::Forbidden(_))));
}
