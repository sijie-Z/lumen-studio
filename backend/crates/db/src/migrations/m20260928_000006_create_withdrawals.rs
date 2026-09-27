use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Withdrawals::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Withdrawals::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Withdrawals::CreatorId).integer().not_null())
                    .col(
                        ColumnDef::new(Withdrawals::Amount)
                            .decimal_len(10, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Withdrawals::Fee)
                            .decimal_len(10, 2)
                            .not_null()
                            .default("0.00"),
                    )
                    .col(ColumnDef::new(Withdrawals::ActualAmount).decimal_len(10, 2))
                    .col(
                        ColumnDef::new(Withdrawals::Status)
                            .string()
                            .string_len(20)
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(Withdrawals::AccountInfo).json())
                    .col(ColumnDef::new(Withdrawals::ReviewedBy).integer())
                    .col(ColumnDef::new(Withdrawals::ReviewedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Withdrawals::ReviewNote).text())
                    .col(ColumnDef::new(Withdrawals::CompletedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Withdrawals::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_withdrawals_creator")
                            .from(Withdrawals::Table, Withdrawals::CreatorId)
                            .to(CreatorProfiles::Table, CreatorProfiles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_withdrawals_reviewer")
                            .from(Withdrawals::Table, Withdrawals::ReviewedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_withdrawals_creator")
                    .table(Withdrawals::Table)
                    .col(Withdrawals::CreatorId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_withdrawals_status")
                    .table(Withdrawals::Table)
                    .col(Withdrawals::Status)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Withdrawals::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Withdrawals {
    Table,
    Id,
    CreatorId,
    Amount,
    Fee,
    ActualAmount,
    Status,
    AccountInfo,
    ReviewedBy,
    ReviewedAt,
    ReviewNote,
    CompletedAt,
    CreatedAt,
}

#[derive(Iden)]
enum CreatorProfiles {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
