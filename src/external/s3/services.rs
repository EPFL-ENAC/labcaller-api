use crate::config::Config;
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
