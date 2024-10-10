use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "file_objects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub created_on: DateTimeWithTimeZone,
    pub filename: Option<String>,
    pub size_bytes: Option<i64>,
    pub upload_id: Option<String>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub parts: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::uploads::associations::db::Entity")]
    FileObjectAssociations,
}

impl Related<crate::uploads::associations::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FileObjectAssociations.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
