mod auth;
mod ai;
mod health;
mod uploads;
mod works;
mod creators;
mod service_routes;
mod appointments;
mod admin;

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
        .nest("/api/v1", admin::router())
        .nest("/api/v1", works::router())
        .nest("/api/v1", uploads::router())
}
