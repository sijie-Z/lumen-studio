mod middleware;
mod routes;
mod state;

use anyhow::{bail, Context};
use axum::http::{HeaderValue, Uri};
use axum::middleware as axum_middleware;
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

    let jwt_secret = validate_jwt_secret(std::env::var("JWT_SECRET").ok().as_deref())?;
    let auth = AuthService::new(db.clone(), jwt_secret, 15 * 60, 7 * 24 * 60 * 60);
    let services_catalog = services::service_catalog::ServiceCatalog::new(db.clone());
    services_catalog
        .ensure_default_types()
        .await
        .context("failed to seed service types")?;
    services::seed::seed_demo_data(&db)
        .await
        .context("failed to seed demo data")?;
    let rate_limiter = middleware::rate_limit::from_env()?;
    let app_state = AppState {
        db: db.clone(),
        auth,
        chat: ai::ChatClient::from_env(),
        works: services::work_service::WorkService::new(db.clone()),
        creators: services::creator_service::CreatorService::new(db.clone()),
        services: services_catalog,
        appointments: services::appointment_service::AppointmentService::new(db.clone()),
        payments: services::payment_service::PaymentService::new(db.clone()),
        notifications: services::notification_service::NotificationService::new(db.clone()),
        reviews: services::review_service::ReviewService::new(db.clone()),
        withdrawals: services::withdrawal_service::WithdrawalService::new(db.clone()),
        admin: services::admin_service::AdminService::new(db),
        rate_limiter,
    };

    let cors = cors_layer_from_env()?;
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".into());
    let app = Router::new()
        .merge(routes::router())
        .nest_service("/uploads", ServeDir::new(&upload_dir))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::rate_limit::enforce,
        ))
        .with_state(app_state);

    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| ([127, 0, 0, 1], 8080).into());
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("failed to bind TCP listener")?;
    tracing::info!("Photography AI Platform API listening on {addr}");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

async fn connect_database(url: &str) -> anyhow::Result<DatabaseConnection> {
    let db = db::connect(url)
        .await
        .context("failed to connect to database")?;
    Ok(db)
}

fn validate_jwt_secret(secret: Option<&str>) -> anyhow::Result<String> {
    let secret = secret.map(str::trim).filter(|value| !value.is_empty());
    let Some(secret) = secret else {
        bail!("JWT_SECRET must be set to a random string of at least 32 characters");
    };
    if secret.len() < 32 {
        bail!("JWT_SECRET must be set to a random string of at least 32 characters");
    }
    Ok(secret.to_string())
}

fn cors_layer_from_env() -> anyhow::Result<CorsLayer> {
    let origins = parse_cors_origins(std::env::var("CORS_ORIGINS").ok().as_deref())?;
    Ok(CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers(Any))
}

fn parse_cors_origins(raw: Option<&str>) -> anyhow::Result<Vec<HeaderValue>> {
    let raw = raw.map(str::trim).filter(|value| !value.is_empty());
    let values = match raw {
        Some(raw) => raw
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>(),
        None => vec!["http://localhost:5173", "http://127.0.0.1:5173"],
    };
    if values.is_empty() {
        bail!("CORS_ORIGINS must contain at least one origin when set");
    }
    values
        .into_iter()
        .map(|origin| {
            let uri = origin
                .parse::<Uri>()
                .with_context(|| format!("invalid CORS origin: {origin}"))?;
            if !matches!(uri.scheme_str(), Some("http" | "https")) || uri.authority().is_none() {
                bail!("CORS origins must use http:// or https://: {origin}");
            }
            origin
                .parse::<HeaderValue>()
                .with_context(|| format!("invalid CORS origin: {origin}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_cors_origins, validate_jwt_secret};

    #[test]
    fn jwt_secret_requires_a_strong_value() {
        assert!(validate_jwt_secret(None).is_err());
        assert!(validate_jwt_secret(Some("")).is_err());
        assert!(validate_jwt_secret(Some("short-secret")).is_err());

        let secret = "0123456789abcdef0123456789abcdef";
        assert_eq!(validate_jwt_secret(Some(secret)).unwrap(), secret);
    }

    #[test]
    fn cors_origins_default_to_local_dev_and_reject_invalid_values() {
        let defaults = parse_cors_origins(None).unwrap();
        assert_eq!(defaults.len(), 2);
        assert_eq!(defaults[0], "http://localhost:5173");

        let configured =
            parse_cors_origins(Some("https://example.com, https://admin.example.com")).unwrap();
        assert_eq!(configured.len(), 2);
        assert_eq!(configured[1], "https://admin.example.com");

        assert!(parse_cors_origins(Some("not a valid origin")).is_err());
    }
}
