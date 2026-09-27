use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Notifications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Notifications::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Notifications::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(Notifications::Type)
                            .string()
                            .string_len(40)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Notifications::Title)
                            .string()
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Notifications::Content).text())
                    .col(
                        ColumnDef::new(Notifications::Priority)
                            .string()
                            .string_len(10)
                            .not_null()
                            .default("normal"),
                    )
                    .col(
                        ColumnDef::new(Notifications::ActionUrl)
                            .string()
                            .string_len(255),
                    )
                    .col(
                        ColumnDef::new(Notifications::IsRead)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Notifications::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_notifications_user")
                            .from(Notifications::Table, Notifications::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_notifications_user")
                    .table(Notifications::Table)
                    .col(Notifications::UserId)
                    .col(Notifications::IsRead)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Notifications::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Notifications {
    Table,
    Id,
    UserId,
    Type,
    Title,
    Content,
    Priority,
    ActionUrl,
    IsRead,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
