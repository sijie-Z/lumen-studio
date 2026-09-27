mod admin;
mod ai;
mod appointments;
mod auth;
mod creators;
mod health;
mod payments;
mod reviews;
mod service_routes;
mod uploads;
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
        .nest("/api/v1", reviews::router())
        .nest("/api/v1", admin::router())
        .nest("/api/v1", works::router())
        .nest("/api/v1", uploads::router())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use services::{
        admin_service::AdminService, appointment_service::AppointmentService,
        auth_service::AuthService, creator_service::CreatorService,
        payment_service::PaymentService, review_service::ReviewService,
        service_catalog::ServiceCatalog, work_service::WorkService,
    };
    use tower::ServiceExt;

    async fn test_app() -> Router {
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
            reviews: ReviewService::new(db.clone()),
            admin: AdminService::new(db),
        };
        router().with_state(state)
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
}
