use super::db::ActiveModel;
use super::db::ServiceName;
use chrono::NaiveDateTime;
use sea_orm::JsonValue;
use sea_orm::{FromQueryResult, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct Service {
    pub id: Uuid,
    pub service_name: ServiceName,
    pub is_online: bool,
    pub details: Option<JsonValue>,
    pub time_utc: NaiveDateTime,
}

impl From<super::db::Model> for Service {
    fn from(model: super::db::Model) -> Self {
        Self {
            id: model.id,
            service_name: model.service_name,
            is_online: model.is_online,
            details: model.details,
            time_utc: model.time_utc,
        }
    }
}

#[derive(ToSchema, Deserialize, Serialize)]
pub struct ServiceCreate {
    pub service_name: ServiceName,
    pub is_online: bool,
    pub details: Option<JsonValue>,
}

impl From<ServiceCreate> for ActiveModel {
    fn from(create: ServiceCreate) -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            service_name: Set(create.service_name),
            is_online: Set(create.is_online),
            details: Set(create.details),
            time_utc: Set(chrono::Utc::now().naive_utc()),
        }
    }
}
