use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Payments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Payments::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Payments::AppointmentId).integer())
                    .col(ColumnDef::new(Payments::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(Payments::Amount)
                            .decimal_len(10, 2)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Payments::Method).string().string_len(20))
                    .col(
                        ColumnDef::new(Payments::Status)
                            .string()
                            .string_len(20)
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(Payments::PaymentType)
                            .string()
                            .string_len(20)
                            .not_null()
                            .default("appointment"),
                    )
                    .col(ColumnDef::new(Payments::TxId).string().string_len(64))
                    .col(ColumnDef::new(Payments::ExpireTime).timestamp_with_time_zone())
                    .col(ColumnDef::new(Payments::PaymentChannel).string().string_len(20))
                    .col(ColumnDef::new(Payments::RefundAmount).decimal_len(10, 2))
                    .col(ColumnDef::new(Payments::RefundReason).text())
                    .col(
                        ColumnDef::new(Payments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_payments_user")
                            .from(Payments::Table, Payments::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Reviews::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Reviews::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Reviews::AppointmentId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Reviews::UserId).integer().not_null())
                    .col(ColumnDef::new(Reviews::CreatorId).integer().not_null())
                    .col(
                        ColumnDef::new(Reviews::Rating)
                            .decimal_len(2, 1)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Reviews::Content).text())
                    .col(ColumnDef::new(Reviews::Images).json())
                    .col(
                        ColumnDef::new(Reviews::IsAnonymous)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Reviews::PhotographerReply).text())
                    .col(ColumnDef::new(Reviews::RepliedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Reviews::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_reviews_user")
                            .from(Reviews::Table, Reviews::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_reviews_creator")
                            .from(Reviews::Table, Reviews::CreatorId)
                            .to(CreatorProfiles::Table, CreatorProfiles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_payments_user")
                    .table(Payments::Table)
                    .col(Payments::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_reviews_creator")
                    .table(Reviews::Table)
                    .col(Reviews::CreatorId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Reviews::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Payments::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Payments {
    Table,
    Id,
    AppointmentId,
    UserId,
    Amount,
    Method,
    Status,
    PaymentType,
    TxId,
    ExpireTime,
    PaymentChannel,
    RefundAmount,
    RefundReason,
    CreatedAt,
}

#[derive(Iden)]
enum Reviews {
    Table,
    Id,
    AppointmentId,
    UserId,
    CreatorId,
    Rating,
    Content,
    Images,
    IsAnonymous,
    PhotographerReply,
    RepliedAt,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum CreatorProfiles {
    Table,
    Id,
}
