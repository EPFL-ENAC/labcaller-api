use super::models::HealthCheck;
use crate::common::models::UIConfiguration;
use crate::external::k8s::services::get_pods;
use axum::{http::StatusCode, Json};

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
pub async fn healthz() -> (StatusCode, Json<HealthCheck>) {
    // Get health of the API.
    match get_pods().await {
        Ok(_) => {
            return (
                StatusCode::OK,
                Json(HealthCheck {
                    status: "ok".to_string(),
                }),
            )
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HealthCheck {
                    status: "error".to_string(),
                }),
            )
        }
    };
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
