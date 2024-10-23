use chrono::NaiveDateTime;
use sea_orm::FromQueryResult;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct UploadReadOne {
    id: Uuid,
    created_on: NaiveDateTime,
    filename: String,
    size_bytes: i64,
    all_parts_received: bool,
    last_part_received: Option<NaiveDateTime>,
    processing_message: Option<String>,
}

impl From<super::db::Model> for UploadReadOne {
    fn from(model: super::db::Model) -> Self {
        Self {
            id: model.id,
            created_on: model.created_on,
            filename: model.filename,
            size_bytes: model.size_bytes,
            all_parts_received: model.all_parts_received,
            last_part_received: model.last_part_received,
            processing_message: model.processing_message,
        }
    }
}
