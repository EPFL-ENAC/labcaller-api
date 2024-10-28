use crate::config::Config;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(ToSchema, Deserialize, Default)]
pub struct FilterOptions {
    pub filter: Option<String>, // JSON-encoded filter
    pub range: Option<String>,  // range in the format "[0,24]"
    pub sort: Option<String>,   // sort in the format '["id", "ASC"]'
}

#[derive(ToSchema, Deserialize, Serialize, Default)]
pub struct Keycloak {
    pub client_id: String,
    pub realm: String,
    pub url: String,
}

#[derive(ToSchema, Deserialize, Serialize, Default)]
pub struct UIConfiguration {
    pub keycloak: Keycloak,
    pub deployment: String,
}

impl UIConfiguration {
    pub fn new() -> Self {
        let config: Config = Config::from_env();
        Self {
            keycloak: Keycloak {
                client_id: config.keycloak_ui_id,
                realm: config.keycloak_realm,
                url: config.keycloak_url,
            },
            deployment: config.deployment,
        }
    }
}

#[derive(ToSchema, Deserialize, Serialize)]
pub struct HealthCheck {
    pub status: String,
}

#[derive(ToSchema, Deserialize, Serialize)]
pub struct ServiceStatus {
    pub s3_status: bool,
    pub kubernetes_status: bool,
}
