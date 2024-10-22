use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the InputObjects table
        manager
            .create_table(
                Table::create()
                    .table(FileObjects::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(FileObjects::Id).uuid().primary_key())
                    .col(
                        ColumnDef::new(FileObjects::CreatedOn)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(FileObjects::Filename).string())
                    .col(ColumnDef::new(FileObjects::SizeBytes).big_integer())
                    .col(ColumnDef::new(FileObjects::UploadId).string())
                    .col(ColumnDef::new(FileObjects::Parts).json_binary())
                    .col(ColumnDef::new(FileObjects::AllPartsReceived).boolean())
                    .col(ColumnDef::new(FileObjects::LastPartReceived).date_time())
                    .to_owned(),
            )
            .await?;

        // Create indexes separately for the FileObjects table
        manager
            .create_index(
                Index::create()
                    .name("idx_file_obj_id")
                    .table(FileObjects::Table)
                    .col(FileObjects::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_file_obj_filename")
                    .table(FileObjects::Table)
                    .col(FileObjects::Filename)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_file_obj_upload_id")
                    .table(FileObjects::Table)
                    .col(FileObjects::UploadId)
                    .to_owned(),
            )
            .await?;

        // Create the InputObjectAssociations table
        manager
            .create_table(
                Table::create()
                    .table(FileObjectAssociations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FileObjectAssociations::Iterator)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(FileObjectAssociations::InputObjectId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FileObjectAssociations::SubmissionId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk_input_object_id")
                            .from_tbl(FileObjectAssociations::Table)
                            .from_col(FileObjectAssociations::InputObjectId)
                            .to_tbl(FileObjects::Table)
                            .to_col(FileObjects::Id),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk_submission_id")
                            .from_tbl(FileObjectAssociations::Table)
                            .from_col(FileObjectAssociations::SubmissionId)
                            .to_tbl(Submissions::Table)
                            .to_col(Submissions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes and unique constraints for the InputObjectAssociations table
        manager
            .create_index(
                Index::create()
                    .name("idx_no_same_link_constraint")
                    .table(FileObjectAssociations::Table)
                    .unique()
                    .col(FileObjectAssociations::InputObjectId)
                    .col(FileObjectAssociations::SubmissionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_association_submission_id")
                    .table(FileObjectAssociations::Table)
                    .col(FileObjectAssociations::SubmissionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_association_input_object_id")
                    .table(FileObjectAssociations::Table)
                    .col(FileObjectAssociations::InputObjectId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the tables in reverse order to respect foreign key dependencies
        manager
            .drop_table(
                Table::drop()
                    .table(FileObjectAssociations::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(FileObjects::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum FileObjects {
    Table,
    Id,
    Filename,
    SizeBytes,
    UploadId,
    Parts,
    CreatedOn,
    AllPartsReceived,
    LastPartReceived,
}

#[derive(DeriveIden)]
enum FileObjectAssociations {
    Table,
    Iterator,
    InputObjectId,
    SubmissionId,
}

#[derive(DeriveIden)]
enum Submissions {
    Table,
    Id,
}
