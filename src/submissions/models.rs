use chrono::NaiveDate;
use chrono::NaiveDateTime;
use sea_orm::FromQueryResult;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct Submission {
    id: Uuid,
    name: String,
    processing_has_started: bool,
    processing_success: bool,
    comment: Option<String>,
    created_on: Option<NaiveDate>,
    last_updated: NaiveDateTime,
}

impl From<super::db::Model> for Submission {
    fn from(model: super::db::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            processing_has_started: model.processing_has_started,
            processing_success: model.processing_success,
            comment: model.comment,
            created_on: model.created_on,
            last_updated: model.last_updated,
        }
    }
}
