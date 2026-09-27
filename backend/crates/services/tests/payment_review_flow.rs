use chrono::{Duration, Utc};
use common::AppError;
use db::entities::{
    appointment as appointment_entity, creator_profile as creator_entity,
    service as service_entity, service_type as service_type_entity, user as user_entity,
};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set};
use services::payment_service::PaymentService;
use services::review_service::ReviewService;

async fn insert_user(db: &DatabaseConnection, id: i32, username: &str, balance: Decimal) {
    let now = Utc::now();
    user_entity::ActiveModel {
        id: Set(id),
        username: Set(username.to_string()),
        password_hash: Set("hash".into()),
        nickname: Set(username.to_string()),
        balance: Set(balance),
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

async fn insert_service(db: &DatabaseConnection) {
    let now = Utc::now();
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
        id: Set(1),
        creator_id: Set(1),
        type_id: Set(1),
        title: Set("Portrait session".into()),
        price: Set(Decimal::new(10000, 2)),
        duration: Set(Some(120)),
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

async fn insert_appointment(db: &DatabaseConnection, id: i32, user_id: i32, status: &str) {
    let now = Utc::now();
    let start = now + Duration::days(1);
    appointment_entity::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        creator_id: Set(1),
        service_id: Set(1),
        appointment_date: Set(start.format("%Y-%m-%d").to_string()),
        start_time: Set(start),
        end_time: Set(start + Duration::hours(2)),
        status: Set(status.into()),
        total_price: Set(Decimal::new(10000, 2)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();
}

#[tokio::test]
async fn payment_settlement_and_review_flow() {
    let db = db::connect("sqlite::memory:").await.unwrap();
    db::migrations::run(&db).await.unwrap();

    insert_user(&db, 1, "creator", Decimal::ZERO).await;
    insert_user(&db, 2, "client", Decimal::ZERO).await;
    insert_creator(&db, 1, 1).await;
    insert_service(&db).await;
    insert_appointment(&db, 1, 2, "pending").await;
    insert_appointment(&db, 2, 2, "completed").await;

    let payments = PaymentService::new(db.clone());
    let reviews = ReviewService::new(db.clone());

    let recharge = payments
        .recharge(2, Decimal::new(10000, 2), None)
        .await
        .unwrap();
    assert_eq!(recharge.payment_type, "recharge");
    assert_eq!(recharge.status, "success");

    let paid = payments
        .pay_appointment(2, 1, "balance".into(), None)
        .await
        .unwrap();
    assert_eq!(paid.payment_type, "appointment");
    assert_eq!(paid.payment_channel.as_deref(), Some("escrow"));

    let client = user_entity::Entity::find_by_id(2)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(client.balance, Decimal::ZERO);

    let appointment = appointment_entity::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(appointment.status, "confirmed");

    let duplicate = payments.pay_appointment(2, 1, "balance".into(), None).await;
    assert!(matches!(duplicate, Err(AppError::Conflict(_))));

    let mut active = appointment.into_active_model();
    active.status = Set("completed".into());
    active.update(&db).await.unwrap();

    payments.settle(1).await.unwrap();
    payments.settle(1).await.unwrap();

    let creator_user = user_entity::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(creator_user.balance, Decimal::new(9000, 2));

    let creator = creator_entity::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(creator.total_income, Decimal::new(9000, 2));

    reviews
        .create(2, 1, Decimal::new(50, 1), Some("Excellent".into()), false)
        .await
        .unwrap();
    reviews
        .create(2, 2, Decimal::new(40, 1), Some("Good".into()), false)
        .await
        .unwrap();

    let duplicate_review = reviews.create(2, 1, Decimal::new(50, 1), None, false).await;
    assert!(matches!(duplicate_review, Err(AppError::Conflict(_))));

    let creator = creator_entity::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(creator.avg_rating, Decimal::new(450, 2));
    assert_eq!(reviews.list_for_creator(1).await.unwrap().len(), 2);
}
