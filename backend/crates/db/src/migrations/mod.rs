mod m20260915_000001_create_users;
mod m20260915_000002_create_works;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260915_000001_create_users::Migration),
            Box::new(m20260915_000002_create_works::Migration),
        ]
    }
}

/// Run all pending migrations against the provided connection.
pub async fn run(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}
