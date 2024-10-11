use super::db::{ActiveModel, Entity};
use super::models::ServiceCreate;
use crate::config::Config;
use crate::external::db::ServiceName;
use sea_orm::{Database, DatabaseConnection, EntityTrait};

pub async fn check_external_services() {
    let config = Config::from_env();
    let db: DatabaseConnection = Database::connect(&*config.db_url.as_ref().unwrap())
        .await
        .unwrap();

    // Fetch pods and handle the result
    let pods_result = crate::external::k8s::services::get_pods().await;

    let service: ActiveModel = match pods_result {
        Ok(Some(pods)) => {
            println!("Found {} pods", pods.len());
            let pods_json = serde_json::to_value(pods).unwrap();
            ServiceCreate {
                service_name: ServiceName::RCP,
                is_online: true,
                details: Some(pods_json),
            }
            .into()
        }
        Ok(None) => {
            println!("No pods found.");
            ServiceCreate {
                service_name: ServiceName::RCP,
                is_online: true,
                details: None,
            }
            .into()
        }
        Err(err) => {
            println!("Error with RCP: {}", err);
            let error_json = serde_json::to_value(err.to_string()).unwrap();
            ServiceCreate {
                service_name: ServiceName::RCP,
                is_online: false,
                details: Some(error_json),
            }
            .into()
        }
    };

    // Insert the service record into the database
    Entity::insert(service).exec(&db).await.unwrap();
}
