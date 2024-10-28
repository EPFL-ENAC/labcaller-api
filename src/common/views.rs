use super::models::{HealthCheck, ServiceStatus};
use crate::external::db;
use crate::external::db::ServiceName;
use crate::external::k8s::services::get_pods;
use crate::{common::models::UIConfiguration, external::s3};
use axum::{extract::State, http::StatusCode, Json};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, QueryOrder, QuerySelect,
    Set,
};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/healthz",
    responses(
        (
            status = OK,
            description = "Kubernetes health check",
            body = str,
            content_type = "text/plain"
        )
    )
)]
pub async fn healthz(State(db): State<DatabaseConnection>) -> (StatusCode, Json<HealthCheck>) {
    // Get health of the API.
    match get_pods().await {
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HealthCheck {
                    status: "error".to_string(),
                }),
            )
        }
        _ => {}
    };

    match db.ping().await {
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HealthCheck {
                    status: "error".to_string(),
                }),
            )
        }
        _ => {}
    };

    (
        StatusCode::OK,
        Json(HealthCheck {
            status: "ok".to_string(),
        }),
    )
}

#[utoipa::path(
    get,
    path = "/api/config",
    responses(
        (
            status = OK,
            description = "Web UI configuration",
            body = str,
            content_type = "text/plain"
        )
    )
)]
pub async fn get_ui_config() -> Json<UIConfiguration> {
    Json(UIConfiguration::new())
}

#[utoipa::path(
    get,
    path = "/api/status",
    responses(
        (
            status = OK,
            description = "Status of the API",
            body = str,
            content_type = "text/plain"
        )
    )
)]

pub async fn get_status(State(db): State<DatabaseConnection>) -> Json<ServiceStatus> {
    let config = crate::config::Config::from_env();
    // Check the status of kubernetes and S3 from the last DB entry.
    // This assumes the background runner is updating at frequent intervals
    let k8s = db::Entity::find()
        .filter(db::Column::ServiceName.eq(ServiceName::RCP))
        .order_by_desc(db::Column::TimeUtc)
        .limit(1)
        .all(&db)
        .await
        .unwrap();

    let mut k8s_online = k8s.get(0).unwrap().is_online;

    if (chrono::Utc::now().naive_utc() - k8s.get(0).unwrap().time_utc).num_seconds() as u64
        > config.interval_external_services * 2
    {
        k8s_online = false;
    }

    let s3 = db::Entity::find()
        .filter(db::Column::ServiceName.eq(ServiceName::S3))
        .order_by_desc(db::Column::TimeUtc)
        .limit(1)
        .all(&db)
        .await
        .unwrap();

    let mut s3_online = s3.get(0).unwrap().is_online;

    if (chrono::Utc::now().naive_utc() - s3.get(0).unwrap().time_utc).num_seconds() as u64
        > config.interval_external_services * 2
    {
        s3_online = false;
    }

    Json(ServiceStatus {
        s3_status: s3_online,
        kubernetes_status: k8s_online,
    })
}
