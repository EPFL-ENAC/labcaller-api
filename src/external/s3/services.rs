use crate::config::Config;
use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::{config::Region, Client as S3Client};
use std::sync::Arc;

pub async fn get_client(config: &Config) -> Arc<S3Client> {
    let region = Region::new("us-east-1");
    let credentials = Credentials::new(
        &config.s3_access_key,
        &config.s3_secret_key,
        None,
        None,
        "manual",
    );
    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region.clone())
        .credentials_provider(credentials)
        .endpoint_url(&config.s3_url)
        .load()
        .await;

    Arc::new(S3Client::new(&shared_config))
}

pub async fn get_outputs_from_submission(
    client: &Arc<S3Client>,
    obj: &crate::submissions::db::Model,
) -> Result<Vec<super::models::OutputObject>, Box<dyn std::error::Error>> {
    let config = crate::config::Config::from_env();
    let prefix = format!("{}/outputs/{}/", config.s3_prefix, obj.id);
    let mut outputs: Vec<super::models::OutputObject> = vec![];
    let list = client
        .list_objects()
        .bucket(config.s3_bucket)
        .prefix(prefix.clone())
        .send()
        .await?;

    if let Some(contents) = list.contents {
        for object in contents {
            outputs.push(object.into());
        }
    }

    Ok(outputs)
}

pub async fn delete_output_object(
    client: &Arc<S3Client>,
    object: super::models::OutputObject,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::config::Config::from_env();
    client
        .delete_object()
        .bucket(config.s3_bucket)
        .key(object.key)
        .send()
        .await?;

    Ok(())
}
