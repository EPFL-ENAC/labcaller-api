use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Submissions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Submissions::Id).uuid().primary_key())
                    .col(ColumnDef::new(Submissions::Name).string().unique_key())
                    .col(ColumnDef::new(Submissions::ProcessingHasStarted).boolean())
                    .col(ColumnDef::new(Submissions::ProcessingSuccess).boolean())
                    .col(ColumnDef::new(Submissions::Comment).string())
                    .col(ColumnDef::new(Submissions::CreatedOn).date_time())
                    .col(ColumnDef::new(Submissions::LastUpdated).date_time())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Submissions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Submissions {
    Table,
    Id, // Primary key, UUID
    Name,
    ProcessingHasStarted,
    ProcessingSuccess,
    Comment,
    CreatedOn,
    LastUpdated,
}
