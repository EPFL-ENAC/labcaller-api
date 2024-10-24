use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "file_object_associations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub iterator: i32,
    pub input_object_id: Uuid,
    pub submission_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    // #[sea_orm(
    //     belongs_to = "crate::uploads::db::Entity",
    //     from = "Column::InputObjectId",
    //     to = "crate::uploads::db::Column::Id",
    //     on_update = "NoAction",
    //     on_delete = "NoAction"
    // )]
    FileObjects,
    // #[sea_orm(
    //     belongs_to = "crate::submissions::db::Entity",
    //     from = "Column::SubmissionId",
    //     to = "crate::submissions::db::Column::Id",
    //     on_update = "NoAction",
    //     on_delete = "NoAction"
    // )]
    Submissions,
}

// impl Related<crate::uploads::db::Entity> for Entity {
//     fn to() -> RelationDef {
//         Relation::FileObjects.def()
//     }
// }

// impl Related<crate::submissions::db::Entity> for Entity {
//     fn to() -> RelationDef {
//         Relation::Submissions.def()
//     }
// }

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::FileObjects => Entity::belongs_to(crate::uploads::db::Entity)
                .from(Column::InputObjectId)
                .to(crate::uploads::db::Column::Id)
                .into(),
            Self::Submissions => Entity::belongs_to(crate::submissions::db::Entity)
                .from(Column::SubmissionId)
                .to(crate::submissions::db::Column::Id)
                .into(),
        }
    }
}
impl ActiveModelBehavior for ActiveModel {}
