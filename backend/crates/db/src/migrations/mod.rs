mod m20260915_000001_create_users;
mod m20260915_000002_create_works;
mod m20260916_000003_create_marketplace;
mod m20260916_000004_add_work_category;
mod m20260918_000005_create_payments_reviews;
mod m20260928_000006_create_withdrawals;
mod m20260928_000007_create_notifications;
mod m20260928_000008_payment_consistency;
mod m20260929_000009_create_favorites;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260915_000001_create_users::Migration),
            Box::new(m20260915_000002_create_works::Migration),
            Box::new(m20260916_000003_create_marketplace::Migration),
            Box::new(m20260916_000004_add_work_category::Migration),
            Box::new(m20260918_000005_create_payments_reviews::Migration),
            Box::new(m20260928_000006_create_withdrawals::Migration),
            Box::new(m20260928_000007_create_notifications::Migration),
            Box::new(m20260928_000008_payment_consistency::Migration),
            Box::new(m20260929_000009_create_favorites::Migration),
        ]
    }
}

/// Run all pending migrations against the provided connection.
pub async fn run(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}
