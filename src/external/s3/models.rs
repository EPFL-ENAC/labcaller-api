use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::{DateTime, Utc};
use sea_orm::FromQueryResult;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, FromQueryResult, Debug)]
pub struct OutputObject {
    key: String,
    last_modified: DateTime<Utc>,
    size_bytes: i64,
}

impl From<aws_sdk_s3::types::Object> for OutputObject {
    fn from(model: aws_sdk_s3::types::Object) -> Self {
        Self {
            key: model.key.unwrap(),
            last_modified: model.last_modified.unwrap().to_chrono_utc().unwrap(),
            size_bytes: model.size.unwrap(),
        }
    }
}

#[derive(ToSchema, Serialize, FromQueryResult, Debug)]
pub struct OutputObjectResponse {
    // Let's not show the full key path through the API
    filename: String,
    last_modified: DateTime<Utc>,
    size_bytes: i64,
}

impl From<OutputObject> for OutputObjectResponse {
    fn from(model: OutputObject) -> Self {
        // Filename is the split of the key by the last '/'
        let filename = model.key.split('/').last().unwrap().to_string();
        Self {
            last_modified: model.last_modified,
            filename: filename,
            size_bytes: model.size_bytes,
        }
    }
}