use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::sea_query::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the enum type 'service_name' with values 'rcp' and 's3'
        manager
            .create_type(
                Type::create()
                    .as_enum(ServiceName::ServiceName)
                    .values([ServiceName::RCP, ServiceName::S3])
                    .to_owned(),
            )
            .await?;

        // Create the 'services' table with 'service_name' column using the enum type
        manager
            .create_table(
                Table::create()
                    .table(Services::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Services::Id).uuid().primary_key())
                    .col(
                        ColumnDef::new(Services::ServiceName)
                            .custom(ServiceName::ServiceName)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Services::IsOnline).boolean().not_null())
                    .col(ColumnDef::new(Services::Details).json_binary())
                    .col(ColumnDef::new(Services::TimeUTC).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the 'services' table
        manager
            .drop_table(Table::drop().table(Services::Table).to_owned())
            .await?;

        // Drop the enum type 'service_name'
        manager
            .drop_type(
                Type::drop()
                    .if_exists()
                    .name(ServiceName::ServiceName)
                    .cascade()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// Define the enum identifiers for the enum type and values
#[derive(Iden)]
enum ServiceName {
    #[iden = "service_name"]
    ServiceName,
    #[iden = "rcp"]
    RCP,
    #[iden = "s3"]
    S3,
}

// Define the Services table and columns
#[derive(DeriveIden)]
enum Services {
    Table,
    Id,          // UUID primary key
    ServiceName, // Name of the service (enum)
    IsOnline,    // True if online, false if offline
    Details,     // JSONB column for additional details (optional)
    TimeUTC,     // Timestamp for checking the status
}
