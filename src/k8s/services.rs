use k8s_openapi::api::core::v1::Pod;
use kube::api::ListParams;
use kube::{api::Api, Client};
use std::error::Error;

pub async fn get_pods_from_namespace() -> Result<(), Box<dyn Error>> {
    // Load the kubeconfig file.
    let config = kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions::default()).await?;
    let client = Client::try_from(config)?;

    // Work with Kubernetes API.
    let pods: Api<Pod> = Api::namespaced(client, "epfl-eceo");
    let lp = ListParams::default();

    // List pods in the namespace.
    for p in pods.list(&lp).await? {
        println!("Found Pod: {}", p.metadata.name.unwrap_or_default());
    }

    Ok(())
}
