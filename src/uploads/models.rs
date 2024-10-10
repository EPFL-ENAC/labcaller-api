use chrono::NaiveDateTime;
use sea_orm::FromQueryResult;
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct UploadReadOne {
    id: Uuid,
    created_on: NaiveDateTime,
    filename: String,
    size_bytes: i64,
    upload_id: Option<String>,
    parts: Value,
}

impl From<super::db::Model> for UploadReadOne {
    fn from(model: super::db::Model) -> Self {
        Self {
            id: model.id,
            created_on: model.created_on,
            filename: model.filename.unwrap_or_else(|| "".to_string()),
            size_bytes: model.size_bytes.unwrap_or(0),
            upload_id: model.upload_id,
            parts: model.parts.unwrap_or(Value::Null),
        }
    }
}
