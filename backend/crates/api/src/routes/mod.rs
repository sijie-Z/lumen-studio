mod auth;
mod ai;
mod health;
mod uploads;

use crate::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .nest("/api/v1/auth", auth::router())
        .nest("/api/v1/ai", ai::router())
        .nest("/api/v1", uploads::router())
}
