use chrono::NaiveDateTime;
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Debug)]
pub struct RunStatus {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub kubernetes_pod_name: Option<String>,
    pub status: Option<String>,
    pub is_running: bool,
    pub is_successful: bool,
    pub is_still_kubernetes_resource: bool,
    pub time_started: Option<String>,
    pub logs: Value,
    pub time_added_utc: NaiveDateTime,
    pub last_updated: NaiveDateTime,
}

impl From<super::db::Model> for RunStatus {
    fn from(model: super::db::Model) -> Self {
        Self {
            id: model.id,
            submission_id: model.submission_id,
            kubernetes_pod_name: model.kubernetes_pod_name,
            status: model.status,
            is_running: model.is_running,
            is_successful: model.is_successful,
            is_still_kubernetes_resource: model.is_still_kubernetes_resource,
            time_started: model.time_started,
            logs: model.logs,
            time_added_utc: model.time_added_utc,
            last_updated: model.last_updated,
        }
    }
}
