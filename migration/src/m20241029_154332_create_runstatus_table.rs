use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::prelude::Json;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the RunStatus table
        manager
            .create_table(
                Table::create()
                    .table(RunStatus::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(RunStatus::Id).uuid().primary_key())
                    .col(ColumnDef::new(RunStatus::SubmissionId).uuid().not_null())
                    .col(ColumnDef::new(RunStatus::KubernetesPodName).string().null())
                    .col(ColumnDef::new(RunStatus::Status).string().null())
                    .col(
                        ColumnDef::new(RunStatus::IsRunning)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(RunStatus::IsSuccessful)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(RunStatus::IsStillKubernetesResource)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(RunStatus::TimeStarted).string().null())
                    .col(
                        ColumnDef::new(RunStatus::Logs)
                            .json()
                            .not_null()
                            .default(Json::Array(vec![])),
                    )
                    .col(
                        ColumnDef::new(RunStatus::TimeAddedUtc)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RunStatus::LastUpdated)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk_run_status_submission_id")
                            .from_tbl(RunStatus::Table)
                            .from_col(RunStatus::SubmissionId)
                            .to_tbl(Submissions::Table)
                            .to_col(Submissions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for the RunStatus table
        manager
            .create_index(
                Index::create()
                    .name("idx_run_status_submission_id")
                    .table(RunStatus::Table)
                    .col(RunStatus::SubmissionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_run_status_kubernetes_pod_name")
                    .table(RunStatus::Table)
                    .col(RunStatus::KubernetesPodName)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_run_status_status")
                    .table(RunStatus::Table)
                    .col(RunStatus::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the RunStatus table in the down migration
        manager
            .drop_table(Table::drop().table(RunStatus::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum RunStatus {
    Table,
    Id,
    SubmissionId,
    KubernetesPodName,
    Status,
    IsRunning,
    IsSuccessful,
    IsStillKubernetesResource,
    TimeStarted,
    Logs,
    TimeAddedUtc,
    LastUpdated,
}

#[derive(DeriveIden)]
enum Submissions {
    Table,
    Id,
}
