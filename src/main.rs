mod common;
mod config;
mod k8s;
mod s3;
mod submissions;

use crate::s3::services::upload_stream;
use axum::{routing::get, Router};
use axum_keycloak_auth::{instance::KeycloakAuthInstance, instance::KeycloakConfig, Url};
use config::Config;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let db: DatabaseConnection = Database::connect(&*config.db_url.as_ref().unwrap())
        .await
        .unwrap();

    if db.ping().await.is_ok() {
        println!("Connected to the database");
    } else {
        println!("Could not connect to the database");
    }

    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    println!("Starting server...");

    let keycloak_auth_instance: Arc<KeycloakAuthInstance> = Arc::new(KeycloakAuthInstance::new(
        KeycloakConfig::builder()
            .server(Url::parse(&config.keycloak_url).unwrap())
            .realm(String::from(&config.keycloak_realm))
            .build(),
    ));
    // get_pods_from_namespace().await.unwrap(); // Gets pods from a namespace
    // get_file_and_upload().await; // Uploads a file to S3
    let app: Router = Router::new()
        .nest(
            "/api/submissions",
            submissions::views::router(db, keycloak_auth_instance),
        )
        .route("/healthz", get(common::views::healthz))
        .route("/api/config", get(common::views::get_ui_config));

    let addr: std::net::SocketAddr = "0.0.0.0:3000".parse().unwrap();
    println!("Listening on {}", addr);

    // Run the server (correct axum usage without `hyper::Server`)
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

pub async fn get_file_and_upload() {
    // Opens file and gets the file name and sends to S3
    let config = Config::from_env();
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];

    let file_name = file_path.split('/').last().unwrap();

    // Open the file and create a stream
    let file = tokio::fs::File::open(file_path)
        .await
        .expect("Failed to open file");
    let stream = tokio::io::BufReader::new(file);

    // Call the upload function with the stream and key
    upload_stream(stream, file_name, &config).await;
}
