use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Services::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Services::Id).uuid().primary_key())
                    .col(
                        ColumnDef::new(Services::ServiceName)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Services::IsOnline).boolean().not_null())
                    .col(ColumnDef::new(Services::Details).json_binary())
                    .col(ColumnDef::new(Services::TimeUTC).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Services::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Services {
    Table,
    Id,          // UUID primary key
    ServiceName, // Name of the service (unique)
    IsOnline,    // True if online, false if offline
    Details,     // JSONB column for additional details (optional)
    TimeUTC,     // Timestamp for checking the status
}
