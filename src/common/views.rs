use super::models::HealthCheck;
use crate::external::k8s::services::get_pods;
use crate::{common::models::UIConfiguration, external::db};
use axum::{extract::State, http::StatusCode, Json};
use sea_orm::DatabaseConnection;

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
