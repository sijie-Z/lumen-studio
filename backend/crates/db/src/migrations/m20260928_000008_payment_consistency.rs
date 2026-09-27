use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Payments::Table)
                    .add_column(
                        ColumnDef::new(Payments::IdempotencyKey)
                            .string()
                            .string_len(128),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_payments_appointment_type")
                    .table(Payments::Table)
                    .col(Payments::AppointmentId)
                    .col(Payments::PaymentType)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_payments_idempotency_key")
                    .table(Payments::Table)
                    .col(Payments::IdempotencyKey)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uq_payments_idempotency_key")
                    .table(Payments::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("uq_payments_appointment_type")
                    .table(Payments::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Payments::Table)
                    .drop_column(Payments::IdempotencyKey)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Payments {
    Table,
    AppointmentId,
    PaymentType,
    IdempotencyKey,
}
