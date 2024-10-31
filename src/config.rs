use dotenvy::dotenv;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;
#[derive(Deserialize)]
pub struct Config {
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub db_prefix: String,
    pub db_url: Option<String>,
    pub app_name: String,
    pub s3_url: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub keycloak_ui_id: String,
    pub keycloak_url: String,
    pub keycloak_realm: String,
    pub deployment: String,
    pub _kube_config: PathBuf,
    pub kube_namespace: String,
    pub interval_external_services: u64,
    pub submission_base_image: String,
    pub submission_base_image_tag: String,

    pub s3_prefix: String,  // Prefix within the bucket, ie. labcaller-dev
    pub pod_prefix: String, // What is prefixed to the pod name, ie. labcaller-dev}
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok(); // Load from .env file if available

        let db_url = env::var("DB_URL").ok();
        let db_prefix = env::var("DB_PREFIX").unwrap_or_else(|_| "postgresql".to_string());
        let s3_prefix = format!(
            "{}-{}",
            env::var("APP_NAME").expect("APP_NAME must be set"),
            env::var("DEPLOYMENT")
                .expect("DEPLOYMENT must be set, this can be local, dev, stage, or prod")
        );
        let pod_prefix = format!(
            "{}-{}",
            env::var("APP_NAME").expect("APP_NAME must be set"),
            env::var("DEPLOYMENT")
                .expect("DEPLOYMENT must be set, this can be local, dev, stage, or prod")
        );

        let config = Config {
            db_host: env::var("DB_HOST").expect("DB_HOST must be set"),
            db_port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap(),
            db_user: env::var("DB_USER").expect("DB_USER must be set"),
            db_password: env::var("DB_PASSWORD").expect("DB_PASSWORD must be set"),
            db_name: env::var("DB_NAME").expect("DB_NAME must be set"),
            app_name: env::var("APP_NAME").expect("APP_NAME must be set"),
            s3_url: env::var("S3_URL").expect("S3_URL must be set"),
            s3_bucket: env::var("S3_BUCKET_ID").expect("S3_BUCKET must be set"),
            s3_access_key: env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY"),
            s3_secret_key: env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY"),
            keycloak_ui_id: env::var("KEYCLOAK_UI_ID").expect("KEYCLOAK_UI_ID must be set"),
            keycloak_url: env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set"),
            keycloak_realm: env::var("KEYCLOAK_REALM").expect("KEYCLOAK_REALM must be set"),
            deployment: env::var("DEPLOYMENT")
                .expect("DEPLOYMENT must be set, this can be local, dev, stage, or prod"),
            _kube_config: env::var("KUBECONFIG")
                .expect("KUBECONFIG must be set")
                .into(),
            kube_namespace: env::var("KUBE_NAMESPACE").expect("KUBE_NAMESPACE must be set"),
            interval_external_services: env::var("INTERVAL_EXTERNAL_SERVICES")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap(),
            submission_base_image: env::var("SUBMISSION_BASE_IMAGE")
                .expect("SUBMISSION_BASE_IMAGE must be set"),
            submission_base_image_tag: env::var("SUBMISSION_BASE_IMAGE_TAG")
                .expect("SUBMISSION_BASE_IMAGE_TAG must be set"),
            db_prefix,
            db_url,
            s3_prefix,
            pod_prefix,
        };

        if config.db_url.is_none() {
            config.form_db_url()
        } else {
            config
        }
    }

    fn form_db_url(mut self) -> Self {
        self.db_url = Some(format!(
            "{}://{}:{}@{}:{}/{}",
            self.db_prefix,
            self.db_user,
            self.db_password,
            self.db_host,
            self.db_port,
            self.db_name,
        ));
        self
    }
}
