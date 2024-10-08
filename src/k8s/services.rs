use crate::config::Config;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams},
    Client,
};
use std::error::Error;

pub async fn get_pods_from_namespace() -> Result<(), Box<dyn Error>> {
    let app_config = Config::from_env();
    let config = kube::Config::infer().await?;

    // Create a client using the kubeconfig.
    let client = Client::try_from(config)?;

    // Specify the namespace to work with.
    let pods: Api<Pod> = Api::namespaced(client, &app_config.kube_namespace);

    // Set up list parameters (can be customized).
    let lp = ListParams::default();

    // Try to list pods in the specified namespace.
    match pods.list(&lp).await {
        Ok(pod_list) => {
            // Iterate and print pod names if successful.
            for p in pod_list {
                println!("Found Pod: {}", p.metadata.name.unwrap_or_default());
            }
        }
        Err(e) => {
            // Handle error, for example if there's an Unauthorized error.
            eprintln!("Error listing pods: {:?}", e);
        }
    }

    Ok(())
}
