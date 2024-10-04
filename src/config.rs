use dotenvy::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub db_prefix: String,
    pub db_url: Option<String>,
    pub s3_url: String,
    pub s3_prefix: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub keycloak_ui_id: String,
    // pub keycloak_api_id: String,
    // pub keycloak_api_secret: String,
    pub keycloak_url: String,
    pub keycloak_realm: String,
    pub deployment: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok(); // Load from .env file if available

        let db_url = env::var("DB_URL").ok();
        let db_prefix = env::var("DB_PREFIX").unwrap_or_else(|_| "postgresql".to_string());

        let config = Config {
            db_host: env::var("DB_HOST").expect("DB_HOST must be set"),
            db_port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap(),
            db_user: env::var("DB_USER").expect("DB_USER must be set"),
            db_password: env::var("DB_PASSWORD").expect("DB_PASSWORD must be set"),
            db_name: env::var("DB_NAME").expect("DB_NAME must be set"),
            s3_url: env::var("S3_URL").expect("S3_URL must be set"),
            s3_prefix: env::var("S3_PREFIX").expect("S3_PREFIX must be set"),
            s3_bucket: env::var("S3_BUCKET_ID").expect("S3_BUCKET must be set"),
            s3_access_key: env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY"),
            s3_secret_key: env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY"),
            keycloak_ui_id: env::var("KEYCLOAK_UI_ID").expect("KEYCLOAK_UI_ID must be set"),
            // keycloak_api_id: env::var("KEYCLOAK_API_ID").expect("KEYCLOAK_API_ID must be set"),
            // keycloak_api_secret: env::var("KEYCLOAK_API_SECRET")
            // .expect("KEYCLOAK_API_SECRET must be set"),
            keycloak_url: env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set"),
            keycloak_realm: env::var("KEYCLOAK_REALM").expect("KEYCLOAK_REALM must be set"),
            deployment: env::var("DEPLOYMENT")
                .expect("DEPLOYMENT must be set, this can be local, dev, stage, or prod"),
            db_prefix,
            db_url,
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
