use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use sea_orm::RelationTrait;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, ToSchema)]
#[sea_orm(table_name = "submissions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    pub processing_has_started: bool,
    pub processing_success: bool,
    pub comment: Option<String>,
    pub created_on: NaiveDateTime,
    pub last_updated: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::uploads::db::Entity")]
    FileObjectAssociations,
    #[sea_orm(has_many = "crate::submissions::run_status::db::Entity")]
    RunStatus,
}

impl Related<crate::uploads::db::Entity> for Entity {
    fn to() -> RelationDef {
        crate::uploads::associations::db::Relation::FileObjects.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            crate::uploads::associations::db::Relation::Submissions
                .def()
                .rev(),
        )
    }
}

// Implement the Related trait for RunStatus to complete the relationship
impl Related<crate::submissions::run_status::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RunStatus.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
