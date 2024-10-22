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

#[axum::debug_handler]
pub async fn handle_tus_hooks(
    // State(db): State<DatabaseConnection>,
    Json(payload): Json<crate::external::tus::models::EventPayload>,
) -> impl IntoResponse {
    // println!("TUS hook payload: {:#?}", payload);
    match payload.event_type {
        crate::external::tus::models::EventType::PreCreate => {
            println!("Pre-create event");
            return (StatusCode::NOT_FOUND, "Not Found: ".to_string());
        }
        crate::external::tus::models::EventType::PostReceive => {
            println!("Post-receive event");
            return (StatusCode::NOT_FOUND, "Not Found: ".to_string());
        }
        crate::external::tus::models::EventType::PostCreate => {
            println!("Post-create event");
            return (StatusCode::NOT_FOUND, format!("Not Found: "));
        }
        crate::external::tus::models::EventType::PreFinish => {
            println!("Pre-finish event");
            return (StatusCode::NOT_FOUND, "Not Found: ".to_string());
        }
        crate::external::tus::models::EventType::PostFinish => {
            println!("Post-finish event");
            return (StatusCode::NOT_FOUND, "Not Found: ".to_string());
        }
        _ => {
            println!("Unknown event type");
            return (StatusCode::NOT_FOUND, "Not Found: ".to_string());
        }
    }

    (StatusCode::OK, "".to_string())
}
