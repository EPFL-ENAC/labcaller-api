use chrono::NaiveDateTime;
use sea_orm::{entity::prelude::*, JsonValue};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, ToSchema, Serialize, Deserialize)]
#[sea_orm(table_name = "services")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub service_name: ServiceName,
    pub is_online: bool,
    pub details: Option<JsonValue>,
    pub time_utc: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, ToSchema, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "service_name")]
pub enum ServiceName {
    #[sea_orm(string_value = "rcp")]
    RCP,
    #[sea_orm(string_value = "s3")]
    S3,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
