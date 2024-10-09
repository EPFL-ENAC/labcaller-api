use super::db::{ActiveModel, Entity};
use super::models::{Service, ServiceCreate};
use crate::config::Config;
use crate::services::k8s::models::PodName;
use sea_orm::{Database, DatabaseConnection, EntityTrait, JsonValue};
use uuid::Uuid;

pub async fn check_external_services() {
    // Root function to check extern services (S3, K8s, etc), update their
    // status in the database and update any other services if necessary
    let config = Config::from_env();
    let db: DatabaseConnection = Database::connect(&*config.db_url.as_ref().unwrap())
        .await
        .unwrap();

    let pods: Option<Vec<PodName>> = crate::services::k8s::services::get_pods(true)
        .await
        .unwrap();

    // Handle the Option and iterate through the pods
    if let Some(pods) = pods {
        println!("Found {} pods", pods.len());
        // Serialise the pods into a JSON to put into DB
        let pods_json: JsonValue = serde_json::to_value(pods).unwrap();

        let service: ActiveModel = ServiceCreate {
            service_name: "rcp".to_string(),
            is_online: true,
            details: Some(pods_json),
        }
        .into();
        Entity::insert(service)
            .exec(&db)
            .await
            .expect("Failed to insert service");
    } else {
        println!("No pods found.");
    }
}
