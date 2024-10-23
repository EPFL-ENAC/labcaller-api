use crate::common::models::UIConfiguration;
use axum::{debug_handler, http::StatusCode, response::IntoResponse, Json};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Status;
use sea_orm::query::*;
use serde_json;

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
pub async fn healthz() -> &'static str {
    // Get health of the API.
    "ok"
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
