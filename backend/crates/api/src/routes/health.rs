use crate::state::AppState;
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<Value> {
    Json(json!({
        "code": 200,
        "message": "success",
        "data": {
            "status": "ok"
        }
    }))
}
