use chrono::{Duration, Utc};
use db::entities::{
    appointment as appointment_entity, creator_profile as creator_entity,
    payment as payment_entity, service as service_entity, service_type as service_type_entity,
    user as user_entity,
};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use services::appointment_service::{AppointmentService, CreateAppointmentInput};
use services::payment_service::PaymentService;

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

async fn insert_service(db: &DatabaseConnection, id: i32, creator_id: i32) {
    let now = Utc::now();
    let type_id = id;
    service_type_entity::ActiveModel {
        id: Set(type_id),
        name: Set(format!("type-{id}")),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    service_entity::ActiveModel {
        id: Set(id),
        creator_id: Set(creator_id),
        type_id: Set(type_id),
        title: Set(format!("Service {id}")),
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

async fn setup() -> DatabaseConnection {
    let db = db::connect("sqlite::memory:").await.unwrap();
    db::migrations::run(&db).await.unwrap();
    insert_user(&db, 1, "creator", Decimal::ZERO).await;
    insert_user(&db, 2, "client", Decimal::new(10000, 2)).await;
    insert_creator(&db, 1, 1).await;
    insert_service(&db, 1, 1).await;
    db
}

fn appointment_input(service_id: i32, start: chrono::DateTime<Utc>) -> CreateAppointmentInput {
    CreateAppointmentInput {
        service_id,
        start_time: start,
        end_time: start + Duration::hours(2),
        location: None,
        notes: None,
    }
}

async fn balance(db: &DatabaseConnection, user_id: i32) -> Decimal {
    user_entity::Entity::find_by_id(user_id)
        .one(db)
        .await
        .unwrap()
        .unwrap()
        .balance
}

async fn appointment_status(db: &DatabaseConnection, appointment_id: i32) -> String {
    appointment_entity::Entity::find_by_id(appointment_id)
        .one(db)
        .await
        .unwrap()
        .unwrap()
        .status
}

async fn payment_count(db: &DatabaseConnection, appointment_id: i32, payment_type: &str) -> u64 {
    payment_entity::Entity::find()
        .filter(payment_entity::Column::AppointmentId.eq(appointment_id))
        .filter(payment_entity::Column::PaymentType.eq(payment_type))
        .count(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn paid_cancellation_refunds_customer_and_writes_refund() {
    let db = setup().await;
    let appointments = AppointmentService::new(db.clone());
    let payments = PaymentService::new(db.clone());
    let start = Utc::now() + Duration::days(1);

    let appointment = appointments
        .create(2, appointment_input(1, start))
        .await
        .unwrap();
    payments
        .pay_appointment(2, appointment.id, "balance".into(), None)
        .await
        .unwrap();
    assert_eq!(balance(&db, 2).await, Decimal::ZERO);

    appointments
        .transition(2, None, appointment.id, "cancelled".into())
        .await
        .unwrap();

    assert_eq!(appointment_status(&db, appointment.id).await, "refunded");
    assert_eq!(balance(&db, 2).await, Decimal::new(10000, 2));
    assert_eq!(payment_count(&db, appointment.id, "refund").await, 1);
}

#[tokio::test]
async fn unpaid_cancellation_does_not_refund() {
    let db = setup().await;
    let appointments = AppointmentService::new(db.clone());
    let start = Utc::now() + Duration::days(1);

    let appointment = appointments
        .create(2, appointment_input(1, start))
        .await
        .unwrap();
    appointments
        .transition(2, None, appointment.id, "cancelled".into())
        .await
        .unwrap();

    assert_eq!(appointment_status(&db, appointment.id).await, "cancelled");
    assert_eq!(balance(&db, 2).await, Decimal::new(10000, 2));
    assert_eq!(payment_count(&db, appointment.id, "refund").await, 0);
}

#[tokio::test]
async fn refunded_status_cannot_be_set_directly() {
    let db = setup().await;
    let appointments = AppointmentService::new(db.clone());
    let payments = PaymentService::new(db.clone());
    let start = Utc::now() + Duration::days(1);

    let appointment = appointments
        .create(2, appointment_input(1, start))
        .await
        .unwrap();
    payments
        .pay_appointment(2, appointment.id, "balance".into(), None)
        .await
        .unwrap();

    let result = appointments
        .transition(1, Some(1), appointment.id, "refunded".into())
        .await;
    assert!(result.is_err());
    assert_eq!(appointment_status(&db, appointment.id).await, "confirmed");
    assert_eq!(balance(&db, 2).await, Decimal::ZERO);
    assert_eq!(payment_count(&db, appointment.id, "refund").await, 0);
}

#[tokio::test]
async fn completion_and_settlement_are_atomic_and_idempotent() {
    let db = setup().await;
    let appointments = AppointmentService::new(db.clone());
    let payments = PaymentService::new(db.clone());
    let start = Utc::now() + Duration::days(1);

    let appointment = appointments
        .create(2, appointment_input(1, start))
        .await
        .unwrap();
    payments
        .pay_appointment(2, appointment.id, "balance".into(), None)
        .await
        .unwrap();
    appointments
        .transition(1, Some(1), appointment.id, "ongoing".into())
        .await
        .unwrap();
    appointments
        .transition(1, Some(1), appointment.id, "completed".into())
        .await
        .unwrap();

    assert_eq!(balance(&db, 1).await, Decimal::new(9000, 2));
    assert_eq!(payment_count(&db, appointment.id, "settlement").await, 1);

    payments.settle(appointment.id).await.unwrap();
    assert_eq!(balance(&db, 1).await, Decimal::new(9000, 2));
    assert_eq!(payment_count(&db, appointment.id, "settlement").await, 1);
}

#[tokio::test]
async fn completion_rolls_back_when_settlement_fails() {
    let db = setup().await;
    let appointments = AppointmentService::new(db.clone());
    let start = Utc::now() + Duration::days(1);

    let appointment = appointments
        .create(2, appointment_input(1, start))
        .await
        .unwrap();
    appointments
        .transition(1, Some(1), appointment.id, "confirmed".into())
        .await
        .unwrap();
    appointments
        .transition(1, Some(1), appointment.id, "ongoing".into())
        .await
        .unwrap();

    let result = appointments
        .transition(1, Some(1), appointment.id, "completed".into())
        .await;
    assert!(result.is_err(), "completion without payment must fail");
    assert_eq!(appointment_status(&db, appointment.id).await, "ongoing");
    assert_eq!(payment_count(&db, appointment.id, "settlement").await, 0);
}

#[tokio::test]
async fn overlapping_appointments_are_rejected_without_transaction_race() {
    let db = setup().await;
    let appointments = AppointmentService::new(db.clone());
    let start = Utc::now() + Duration::days(1);

    let first = appointments
        .create(2, appointment_input(1, start))
        .await
        .unwrap();

    let sequential = appointments.create(2, appointment_input(1, start)).await;
    assert!(sequential.is_err(), "sequential duplicate slot must fail");
    assert_eq!(
        appointment_entity::Entity::find_by_id(first.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .status,
        "pending"
    );

    let concurrent_start = start + Duration::days(1);
    let left = appointments.clone();
    let right = appointments.clone();
    let (left_result, right_result) = tokio::join!(
        left.create(2, appointment_input(1, concurrent_start)),
        right.create(2, appointment_input(1, concurrent_start))
    );

    assert!(
        left_result.is_ok() ^ right_result.is_ok(),
        "exactly one concurrent booking must succeed"
    );
}

#[tokio::test]
async fn payment_idempotency_key_only_applies_once() {
    let db = setup().await;
    let payments = PaymentService::new(db.clone());

    let first_recharge = payments
        .recharge(2, Decimal::new(5000, 2), Some("recharge-once".into()))
        .await
        .unwrap();
    let second_recharge = payments
        .recharge(2, Decimal::new(5000, 2), Some("recharge-once".into()))
        .await
        .unwrap();

    assert_eq!(first_recharge.id, second_recharge.id);
    assert_eq!(balance(&db, 2).await, Decimal::new(15000, 2));
    assert_eq!(
        payment_entity::Entity::find()
            .filter(payment_entity::Column::IdempotencyKey.eq("recharge-once"))
            .count(&db)
            .await
            .unwrap(),
        1
    );

    let appointments = AppointmentService::new(db.clone());
    let start = Utc::now() + Duration::days(1);
    let appointment = appointments
        .create(2, appointment_input(1, start))
        .await
        .unwrap();

    let first_payment = payments
        .pay_appointment(2, appointment.id, "balance".into(), Some("pay-once".into()))
        .await
        .unwrap();
    let second_payment = payments
        .pay_appointment(2, appointment.id, "balance".into(), Some("pay-once".into()))
        .await
        .unwrap();

    assert_eq!(first_payment.id, second_payment.id);
    assert_eq!(balance(&db, 2).await, Decimal::new(5000, 2));
    assert_eq!(payment_count(&db, appointment.id, "appointment").await, 1);
}
