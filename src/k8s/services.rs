use k8s_openapi::api::core::v1::Pod;
use kube::api::ListParams;
use kube::{
    api::{Api, ResourceExt},
    Client,
};
use std::error::Error;

pub fn pod_unready(p: &Pod) -> Option<String> {
    let status = p.status.as_ref().unwrap();
    if let Some(conds) = &status.conditions {
        let failed = conds
            .iter()
            .filter(|c| c.type_ == "Ready" && c.status == "False")
            .map(|c| c.message.clone().unwrap_or_default())
            .collect::<Vec<_>>()
            .join(",");
        if !failed.is_empty() {
            if p.metadata.labels.as_ref().unwrap().contains_key("job-name") {
                return None; // ignore job based pods, they are meant to exit 0
            }
            return Some(format!("Unready pod {}: {}", p.name_any(), failed));
        }
    }
    None
}

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