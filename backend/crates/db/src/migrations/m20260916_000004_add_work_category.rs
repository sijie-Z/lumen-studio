use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Works::Table)
                    .add_column(ColumnDef::new(Works::Category).string().string_len(50))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Works::Table)
                    .drop_column(Works::Category)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Works {
    Table,
    Category,
}
