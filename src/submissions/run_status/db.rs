use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "run_status")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub submission_id: Uuid,
    pub kubernetes_pod_name: Option<String>,
    pub status: Option<String>,
    pub is_running: bool,
    pub is_successful: bool,
    pub is_still_kubernetes_resource: bool,
    pub time_started: Option<String>,
    pub logs: Json,
    pub time_added_utc: NaiveDateTime,
    pub last_updated: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::submissions::db::Entity",
        from = "Column::SubmissionId",
        to = "crate::submissions::db::Column::Id"
    )]
    Submissions,
}

// Implement the relationship back to Submission
impl Related<crate::submissions::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Submissions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
