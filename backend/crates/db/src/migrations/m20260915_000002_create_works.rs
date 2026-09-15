use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Works::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Works::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Works::UserId).integer().not_null())
                    .col(ColumnDef::new(Works::PortfolioId).integer())
                    .col(
                        ColumnDef::new(Works::ImageUrl)
                            .string()
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Works::Title).string().string_len(255))
                    .col(ColumnDef::new(Works::Description).text())
                    .col(
                        ColumnDef::new(Works::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_works_user")
                            .from(Works::Table, Works::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_works_user_created")
                    .table(Works::Table)
                    .col(Works::UserId)
                    .col(Works::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Works::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Works {
    Table,
    Id,
    UserId,
    PortfolioId,
    ImageUrl,
    Title,
    Description,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
