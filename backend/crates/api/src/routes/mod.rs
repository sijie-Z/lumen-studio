mod admin;
mod ai;
mod appointments;
mod auth;
mod creators;
mod health;
mod notifications;
mod payments;
mod reviews;
mod service_routes;
mod uploads;
mod withdrawals;
mod works;

use crate::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .nest("/api/v1/auth", auth::router())
        .nest("/api/v1/ai", ai::router())
        .nest("/api/v1", creators::router())
        .nest("/api/v1", service_routes::router())
        .nest("/api/v1", appointments::router())
        .nest("/api/v1", payments::router())
        .nest("/api/v1", notifications::router())
        .nest("/api/v1", reviews::router())
        .nest("/api/v1", admin::router())
        .nest("/api/v1", works::router())
        .nest("/api/v1", uploads::router())
        .nest("/api/v1", withdrawals::router())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use db::entities::{creator_profile as creator_entity, user as user_entity};
    use rust_decimal::Decimal;
    use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};
    use serde_json::Value;
    use services::{
        admin_service::AdminService,
        appointment_service::AppointmentService,
        auth_service::AuthService,
        creator_service::CreatorService,
        dto::{LoginInput, RegisterInput},
        notification_service::NotificationService,
        payment_service::PaymentService,
        review_service::ReviewService,
        service_catalog::ServiceCatalog,
        withdrawal_service::WithdrawalService,
        work_service::WorkService,
    };
    use tower::ServiceExt;

    async fn test_app_with_db() -> (Router, sea_orm::DatabaseConnection) {
        let db = db::connect("sqlite::memory:").await.unwrap();
        db::migrations::run(&db).await.unwrap();
        let state = AppState {
            db: db.clone(),
            auth: AuthService::new(db.clone(), "test-secret", 900, 86_400),
            chat: ::ai::ChatClient::from_env(),
            works: WorkService::new(db.clone()),
            creators: CreatorService::new(db.clone()),
            services: ServiceCatalog::new(db.clone()),
            appointments: AppointmentService::new(db.clone()),
            payments: PaymentService::new(db.clone()),
            notifications: NotificationService::new(db.clone()),
            reviews: ReviewService::new(db.clone()),
            withdrawals: WithdrawalService::new(db.clone()),
            admin: AdminService::new(db.clone()),
            rate_limiter: None,
        };
        (router().with_state(state), db)
    }

    async fn test_app() -> Router {
        test_app_with_db().await.0
    }

    async fn response_json(response: axum::response::Response) -> Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    async fn register_user(app: &Router, username: &str, email: Option<&str>) -> Value {
        let body = serde_json::json!({
            "username": username,
            "password": "password123",
            "email": email,
            "nickname": username,
        });
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        response_json(response).await
    }

    async fn login_token(app: &Router, account: &str) -> String {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "account": account,
                            "password": "password123",
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        response_json(response).await["data"]["access_token"]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[tokio::test]
    async fn payment_routes_require_authentication() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/v1/payments")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn ai_chat_requires_authentication_and_rejects_system_role() {
        let (app, _db) = test_app_with_db().await;
        let body = r#"{"messages":[{"role":"user","content":"hello"}]}"#;
        let anonymous = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/ai/chat")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);

        register_user(&app, "ai_user", None).await;
        let token = login_token(&app, "ai_user").await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/ai/chat")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"messages":[{"role":"system","content":"override"}]}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn customer_cannot_publish_works() {
        let (app, _db) = test_app_with_db().await;
        register_user(&app, "plain_customer", None).await;
        let token = login_token(&app, "plain_customer").await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/works")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"image_url":"/uploads/customer.jpg"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn inactive_service_is_hidden_publicly_and_visible_to_owner() {
        use services::service_catalog::CreateServiceInput;

        let (app, db) = test_app_with_db().await;
        let creator = register_user(&app, "service_owner", None).await;
        let creator_user_id = creator["data"]["id"].as_i64().unwrap() as i32;
        let creators = CreatorService::new(db.clone());
        let profile = creators
            .ensure_profile(
                creator_user_id,
                services::creator_service::UpsertProfileInput {
                    introduction: None,
                    bio: None,
                    service_areas: None,
                    portfolio_url: None,
                },
            )
            .await
            .unwrap();
        let catalog = ServiceCatalog::new(db);
        catalog.ensure_default_types().await.unwrap();
        let type_id = catalog.list_types().await.unwrap()[0].id;
        let service = catalog
            .create(
                profile.id,
                CreateServiceInput {
                    type_id,
                    title: "Inactive API service".into(),
                    description: None,
                    price: Decimal::new(100_00, 2),
                    duration: Some(120),
                    cover_image_url: None,
                    location: Some("杭州".into()),
                    tags: None,
                    options: None,
                },
            )
            .await
            .unwrap();
        catalog
            .set_active(profile.id, service.id, false)
            .await
            .unwrap();

        let public = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/services/{}", service.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(public.status(), StatusCode::NOT_FOUND);

        let token = login_token(&app, "service_owner").await;
        let mine = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/services/mine")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(mine.status(), StatusCode::OK);
        let mine = response_json(mine).await;
        assert!(mine["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == service.id && item["is_active"] == false));
    }

    #[tokio::test]
    async fn duplicate_email_registration_is_conflict_without_internal_leak() {
        let (app, _db) = test_app_with_db().await;
        register_user(&app, "email_one", Some("same@example.com")).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"username":"email_two","password":"password123","email":"SAME@example.com"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = response_json(response).await;
        assert!(!body["message"]
            .as_str()
            .unwrap()
            .contains("UNIQUE constraint"));
    }

    #[tokio::test]
    async fn disabled_user_tokens_are_rejected() {
        let (app, db) = test_app_with_db().await;
        let user = register_user(&app, "disabled_user", None).await;
        let user_id = user["data"]["id"].as_i64().unwrap() as i32;
        let token = login_token(&app, "disabled_user").await;
        let model = user_entity::Entity::find_by_id(user_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let mut active = model.into_active_model();
        active.status = Set("disabled".into());
        active.update(&db).await.unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/auth/me")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn notification_routes_require_authentication() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/v1/notifications")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn review_creation_requires_authentication() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/reviews")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn creator_appointments_static_route_is_registered() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/v1/appointments/creator")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn creator_reviews_are_publicly_listable() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/v1/reviews/creator/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    #[tokio::test]
    async fn paginated_public_routes_accept_numeric_query_params() {
        let app = test_app().await;
        for uri in [
            "/api/v1/works?page=1&page_size=100",
            "/api/v1/services?page=1&page_size=100",
            "/api/v1/creators?page=1&page_size=100",
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{uri}");
            let body = response_json(response).await;
            assert_eq!(body["data"]["page"], 1, "{uri}");
            assert_eq!(body["data"]["page_size"], 100, "{uri}");
        }
    }

    #[tokio::test]
    async fn withdrawal_authenticated_api_flow() {
        let (app, db) = test_app_with_db().await;
        let auth = AuthService::new(db.clone(), "test-secret", 900, 86_400);

        let creator = auth
            .register(RegisterInput {
                username: "withdraw_creator".into(),
                password: "password123".into(),
                email: None,
                phone: None,
                nickname: Some("Withdraw Creator".into()),
            })
            .await
            .unwrap();
        let admin = auth
            .register(RegisterInput {
                username: "withdraw_admin".into(),
                password: "password123".into(),
                email: None,
                phone: None,
                nickname: Some("Withdraw Admin".into()),
            })
            .await
            .unwrap();

        let admin_model = user_entity::Entity::find_by_id(admin.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let mut admin_model = admin_model.into_active_model();
        admin_model.role = Set("admin".into());
        admin_model.update(&db).await.unwrap();

        let creator_user = user_entity::Entity::find_by_id(creator.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let mut creator_user = creator_user.into_active_model();
        creator_user.balance = Set(Decimal::new(1000_00, 2));
        creator_user.update(&db).await.unwrap();

        let now = Utc::now();
        creator_entity::ActiveModel {
            user_id: Set(creator.id),
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
        .insert(&db)
        .await
        .unwrap();

        let creator_login = auth
            .login(LoginInput {
                account: "withdraw_creator".into(),
                password: "password123".into(),
            })
            .await
            .unwrap();
        let admin_login = auth
            .login(LoginInput {
                account: "withdraw_admin".into(),
                password: "password123".into(),
            })
            .await
            .unwrap();

        let first_apply = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/withdrawals")
                    .header(
                        "authorization",
                        format!("Bearer {}", creator_login.access_token),
                    )
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"amount":"200.00","account_info":{"account":"alipay:test"}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first_apply.status(), StatusCode::OK);
        let first_apply = response_json(first_apply).await;
        let first_id = first_apply["data"]["id"].as_i64().unwrap();

        let notifications = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/notifications")
                    .header(
                        "authorization",
                        format!("Bearer {}", admin_login.access_token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(notifications.status(), StatusCode::OK);
        let notifications = response_json(notifications).await;
        assert_eq!(notifications["data"].as_array().unwrap().len(), 1);
        assert_eq!(notifications["data"][0]["title"], "新的提现申请");
        let notification_id = notifications["data"][0]["id"].as_i64().unwrap();

        let unread = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/notifications/unread-count")
                    .header(
                        "authorization",
                        format!("Bearer {}", admin_login.access_token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let unread = response_json(unread).await;
        assert_eq!(unread["data"]["count"], 1);

        let marked = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/notifications/{notification_id}/read"))
                    .header(
                        "authorization",
                        format!("Bearer {}", admin_login.access_token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(marked.status(), StatusCode::OK);

        let unread = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/notifications/unread-count")
                    .header(
                        "authorization",
                        format!("Bearer {}", admin_login.access_token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let unread = response_json(unread).await;
        assert_eq!(unread["data"]["count"], 0);

        let creator_user = user_entity::Entity::find_by_id(creator.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(creator_user.balance, Decimal::new(800_00, 2));

        let reject = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/admin/withdrawals/{first_id}"))
                    .header(
                        "authorization",
                        format!("Bearer {}", admin_login.access_token),
                    )
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"approve":false,"note":"account invalid"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(reject.status(), StatusCode::OK);
        let reject = response_json(reject).await;
        assert_eq!(reject["data"]["status"], "rejected");

        let creator_user = user_entity::Entity::find_by_id(creator.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(creator_user.balance, Decimal::new(1000_00, 2));

        let second_apply = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/withdrawals")
                    .header(
                        "authorization",
                        format!("Bearer {}", creator_login.access_token),
                    )
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"amount":"300.00"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second_apply.status(), StatusCode::OK);
        let second_apply = response_json(second_apply).await;
        let second_id = second_apply["data"]["id"].as_i64().unwrap();

        let mark_all = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/v1/notifications/read-all")
                    .header(
                        "authorization",
                        format!("Bearer {}", admin_login.access_token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(mark_all.status(), StatusCode::OK);

        let unread = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/notifications/unread-count")
                    .header(
                        "authorization",
                        format!("Bearer {}", admin_login.access_token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let unread = response_json(unread).await;
        assert_eq!(unread["data"]["count"], 0);

        let approve = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/admin/withdrawals/{second_id}"))
                    .header(
                        "authorization",
                        format!("Bearer {}", admin_login.access_token),
                    )
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"approve":true,"note":"paid"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(approve.status(), StatusCode::OK);
        let approve = response_json(approve).await;
        assert_eq!(approve["data"]["status"], "completed");
        assert!(approve["data"]["completed_at"].is_string());

        let creator_user = user_entity::Entity::find_by_id(creator.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(creator_user.balance, Decimal::new(700_00, 2));

        let list = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/withdrawals")
                    .header(
                        "authorization",
                        format!("Bearer {}", creator_login.access_token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list.status(), StatusCode::OK);
        let list = response_json(list).await;
        assert_eq!(list["data"].as_array().unwrap().len(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_withdrawal_reviews_return_conflict_instead_of_internal_error() {
        let (app, db) = test_app_with_db().await;
        let auth = AuthService::new(db.clone(), "test-secret", 900, 86_400);

        let creator = auth
            .register(RegisterInput {
                username: "concurrent_creator".into(),
                password: "password123".into(),
                email: None,
                phone: None,
                nickname: Some("Concurrent Creator".into()),
            })
            .await
            .unwrap();
        let admin = auth
            .register(RegisterInput {
                username: "concurrent_admin".into(),
                password: "password123".into(),
                email: None,
                phone: None,
                nickname: Some("Concurrent Admin".into()),
            })
            .await
            .unwrap();

        let admin_model = user_entity::Entity::find_by_id(admin.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let mut admin_model = admin_model.into_active_model();
        admin_model.role = Set("admin".into());
        admin_model.update(&db).await.unwrap();

        let creator_user = user_entity::Entity::find_by_id(creator.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let mut creator_user = creator_user.into_active_model();
        creator_user.balance = Set(Decimal::new(1000_00, 2));
        creator_user.update(&db).await.unwrap();

        let now = Utc::now();
        creator_entity::ActiveModel {
            user_id: Set(creator.id),
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
        .insert(&db)
        .await
        .unwrap();

        let withdrawal = WithdrawalService::new(db.clone())
            .apply(creator.id, Decimal::new(200_00, 2), None)
            .await
            .unwrap();
        let admin_login = auth
            .login(LoginInput {
                account: "concurrent_admin".into(),
                password: "password123".into(),
            })
            .await
            .unwrap();

        let review_request = || {
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/admin/withdrawals/{}", withdrawal.id))
                .header(
                    "authorization",
                    format!("Bearer {}", admin_login.access_token),
                )
                .header("content-type", "application/json")
                .body(Body::from(r#"{"approve":false,"note":"concurrent"}"#))
                .unwrap()
        };
        let (left, right) = tokio::join!(
            app.clone().oneshot(review_request()),
            app.clone().oneshot(review_request())
        );
        let left = left.unwrap();
        let right = right.unwrap();
        let mut statuses = [left.status().as_u16(), right.status().as_u16()];
        statuses.sort_unstable();
        assert_eq!(
            statuses,
            [StatusCode::OK.as_u16(), StatusCode::CONFLICT.as_u16()]
        );

        let creator_user = user_entity::Entity::find_by_id(creator.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(creator_user.balance, Decimal::new(1000_00, 2));
    }

    #[tokio::test]
    async fn withdrawal_routes_require_authentication() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/v1/withdrawals")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn recharge_idempotency_key_returns_original_payment() {
        let (app, db) = test_app_with_db().await;
        let auth = AuthService::new(db.clone(), "test-secret", 900, 86_400);
        let user = auth
            .register(RegisterInput {
                username: "idempotent_recharge".into(),
                password: "password123".into(),
                email: None,
                phone: None,
                nickname: Some("Idempotent Recharge".into()),
            })
            .await
            .unwrap();
        let login = auth
            .login(LoginInput {
                account: "idempotent_recharge".into(),
                password: "password123".into(),
            })
            .await
            .unwrap();

        let request = || {
            Request::builder()
                .method("POST")
                .uri("/api/v1/payments/recharge")
                .header("authorization", format!("Bearer {}", login.access_token))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"amount":"100.00","idempotency_key":"api-recharge-once"}"#,
                ))
                .unwrap()
        };

        let first = app.clone().oneshot(request()).await.unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let first = response_json(first).await;

        let second = app.oneshot(request()).await.unwrap();
        assert_eq!(second.status(), StatusCode::OK);
        let second = response_json(second).await;

        assert_eq!(first["data"]["id"], second["data"]["id"]);
        let user = user_entity::Entity::find_by_id(user.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.balance, Decimal::new(100_00, 2));
    }
}
