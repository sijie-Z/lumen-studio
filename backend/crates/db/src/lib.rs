pub mod entities;
pub mod migrations;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

/// Open a database connection. Supports PostgreSQL and SQLite URLs.
pub async fn connect(database_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let mut opts = ConnectOptions::new(database_url);
    opts.max_connections(10)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(600))
        .sqlx_logging(false);

    Database::connect(opts).await
}

/// Open a SQLite database for local development.
pub async fn connect_sqlite(path: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let url = format!("sqlite://{path}?mode=rwc");
    connect(&url).await
}
