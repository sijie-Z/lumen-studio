mod routes;
mod state;
mod middleware;

use anyhow::Context;
use axum::Router;
use sea_orm::DatabaseConnection;
use services::auth_service::AuthService;
use state::AppState;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://photography.db?mode=rwc".into());
    let db = connect_database(&database_url).await?;
    db::migrations::run(&db)
        .await
        .context("failed to run database migrations")?;

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("JWT_SECRET is not set; using development secret");
        "dev-only-secret-change-me".into()
    });
    let auth = AuthService::new(db.clone(), jwt_secret, 15 * 60, 7 * 24 * 60 * 60);
    let app_state = AppState { db, auth };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".into());
    let app = Router::new()
        .merge(routes::router())
        .nest_service("/uploads", ServeDir::new(&upload_dir))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| ([127, 0, 0, 1], 8080).into());
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("failed to bind TCP listener")?;
    tracing::info!("Photography AI Platform API listening on {addr}");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn connect_database(url: &str) -> anyhow::Result<DatabaseConnection> {
    let db = db::connect(url).await.context("failed to connect to database")?;
    Ok(db)
}
