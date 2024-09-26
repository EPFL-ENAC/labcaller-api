mod config;
mod k8s;
mod s3;

use crate::k8s::services::get_pods_from_namespace;
use crate::s3::services::upload_stream;
use config::Config;

use futures::prelude::*;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ResourceExt},
    runtime::{watcher, WatchStreamExt},
    Client,
};
use tracing::*;

#[tokio::main]
async fn main() {
    // Example usage with a file
    // let config = Config::from_env();

    // let args: Vec<String> = std::env::args().collect();
    // if args.len() < 2 {
    //     eprintln!("Usage: {} <file_path>", args[0]);
    //     std::process::exit(1);
    // }
    // let file_path = &args[1];

    // get_file_and_upload(file_path, &config).await;
    // tracing_subscriber::fmt::init();
    // let client = Client::try_default().await.expect("Error creating client");
    // let api = Api::<Pod>::default_namespaced(client);
    // let use_watchlist = std::env::var("WATCHLIST")
    //     .map(|s| s == "1")
    //     .unwrap_or(false);
    // let wc = if use_watchlist {
    //     // requires WatchList feature gate on 1.27 or later
    //     watcher::Config::default().streaming_lists()
    // } else {
    //     watcher::Config::default()
    // };

    // watcher(api, wc)
    //     .applied_objects()
    //     .default_backoff()
    //     .try_for_each(|p| async move {
    //         info!("saw {}", p.name_any());
    //         if let Some(unready_reason) = pod_unready(&p) {
    //             warn!("{}", unready_reason);
    //         }
    //         Ok(())
    //     })
    //     .await
    //     .expect("watch failed");

    get_pods_from_namespace().await.unwrap();
}

pub async fn get_file_and_upload(file_path: &str, config: &Config) {
    // Opens file and gets the file name and sends to S3

    let file_name = file_path.split('/').last().unwrap();

    // Open the file and create a stream
    let file = tokio::fs::File::open(file_path)
        .await
        .expect("Failed to open file");
    let stream = tokio::io::BufReader::new(file);

    // Call the upload function with the stream and key
    upload_stream(stream, file_name, &config).await;
}
