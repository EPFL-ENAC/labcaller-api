use super::db::{ActiveModel, Entity};
use super::models::ServiceCreate;
use crate::config::Config;
use crate::external::db::ServiceName;
use anyhow::{anyhow, Result};
use sea_orm::{Database, DatabaseConnection, EntityTrait};

async fn check_kubernetes() -> Result<serde_json::Value> {
    match crate::external::k8s::services::get_pods().await {
        Ok(pods) => Ok(serde_json::to_value(pods).unwrap()),
        Err(err) => Err(anyhow!(serde_json::to_value(err.to_string()).unwrap())),
    }
}

async fn check_s3(config: &Config) -> Result<serde_json::Value> {
    let s3_client = crate::external::s3::services::get_client(&config).await;

    match s3_client
        .head_bucket()
        .bucket(&config.s3_bucket)
        .send()
        .await
    {
        Ok(_) => Ok(serde_json::to_value("S3 is up").unwrap()),
        Err(err) => Err(anyhow!(serde_json::to_value(err.to_string()).unwrap())),
    }
}

pub async fn check_external_services() {
    let config = Config::from_env();
    let db: DatabaseConnection = Database::connect(&*config.db_url.as_ref().unwrap())
        .await
        .unwrap();

    let k8s: ActiveModel = match check_kubernetes().await {
        Ok(pods) => ServiceCreate {
            service_name: ServiceName::RCP,
            is_online: true,
            details: Some(pods),
        }
        .into(),
        Err(err) => ServiceCreate {
            service_name: ServiceName::RCP,
            is_online: false,
            details: Some(err.to_string().into()),
        }
        .into(),
    };

    Entity::insert(k8s).exec(&db).await.unwrap();

    let s3: ActiveModel = match check_s3(&config).await {
        Ok(s3) => ServiceCreate {
            service_name: ServiceName::S3,
            is_online: true,
            details: Some(s3),
        }
        .into(),
        Err(err) => ServiceCreate {
            service_name: ServiceName::S3,
            is_online: false,
            details: Some(err.to_string().into()),
        }
        .into(),
    };

    Entity::insert(s3).exec(&db).await.unwrap();
}
