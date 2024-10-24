use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "file_objects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub created_on: NaiveDateTime,
    pub filename: String,
    pub size_bytes: i64,
    pub all_parts_received: bool,
    pub last_part_received: Option<NaiveDateTime>,
    pub processing_message: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::submissions::db::Entity")]
    FileObjectAssociations,
}

impl Related<crate::submissions::db::Entity> for Entity {
    fn to() -> RelationDef {
        super::associations::db::Relation::Submissions.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::associations::db::Relation::FileObjects.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
