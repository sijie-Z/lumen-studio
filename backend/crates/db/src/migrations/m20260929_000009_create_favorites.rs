use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Favorites::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Favorites::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Favorites::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(Favorites::TargetType)
                            .string()
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Favorites::TargetId).integer().not_null())
                    .col(
                        ColumnDef::new(Favorites::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_favorites_user")
                            .from(Favorites::Table, Favorites::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_favorites_user_target")
                    .table(Favorites::Table)
                    .col(Favorites::UserId)
                    .col(Favorites::TargetType)
                    .col(Favorites::TargetId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_favorites_user_type")
                    .table(Favorites::Table)
                    .col(Favorites::UserId)
                    .col(Favorites::TargetType)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Favorites::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Favorites {
    Table,
    Id,
    UserId,
    TargetType,
    TargetId,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
