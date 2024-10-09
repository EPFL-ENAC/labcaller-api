use super::models::PodName;
use crate::config::Config;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams},
    config::Kubeconfig,
    Client, Config as KubeConfig,
};
use secrecy::Secret;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize)]
struct TokenResponse {
    id_token: String,
}

async fn refresh_oidc_token(refresh_token: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", "runai-cli"),
    ];

    let res = client
        .post("https://app.run.ai/auth/realms/rcpepfl/protocol/openid-connect/token")
        .form(&params)
        .send()
        .await?;

    if res.status().is_success() {
        println!("Refreshed OIDC token");
        let token_response: TokenResponse = res.json().await?;
        Ok(token_response.id_token)
    } else {
        let error_text = res.text().await?;
        Err(format!("Failed to refresh token: {}", error_text).into())
    }
}

fn extract_refresh_token(kubeconfig: &Kubeconfig) -> Option<String> {
    for named_auth_info in &kubeconfig.auth_infos {
        if let Some(auth_info) = &named_auth_info.auth_info {
            if let Some(auth_provider) = &auth_info.auth_provider {
                if auth_provider.name == "oidc" {
                    // Directly access the config HashMap
                    if let Some(refresh_token) = auth_provider.config.get("refresh-token") {
                        return Some(refresh_token.clone());
                    }
                }
            }
        }
    }
    println!("No refresh token found in kubeconfig.");
    None
}

pub async fn get_pods(match_deployment: bool) -> Result<Option<Vec<PodName>>, Box<dyn Error>> {
    let app_config = Config::from_env();

    // Read and parse the kubeconfig file
    let mut kubeconfig = {
        let mut file = File::open(&app_config._kube_config)?;
        let mut yaml_str = String::new();
        file.read_to_string(&mut yaml_str)?;
        serde_yaml::from_str::<Kubeconfig>(&yaml_str)?
    };

    let refresh_token =
        extract_refresh_token(&kubeconfig).ok_or("Failed to find refresh token in kubeconfig")?;

    // Refresh the OIDC token
    let new_id_token = refresh_oidc_token(&refresh_token).await?;

    // Update the kubeconfig's auth_info
    // Find the current context name
    let current_context_name = kubeconfig
        .current_context
        .clone()
        .ok_or("No current context set in kubeconfig")?;

    // Find the context that matches the current context name
    let context = kubeconfig
        .contexts
        .iter()
        .find(|ctx| ctx.name == current_context_name)
        .ok_or("Failed to find current context in kubeconfig")?;

    // Unwrap the context
    let context_context = context
        .context
        .as_ref()
        .ok_or("Context is missing in NamedContext")?;

    // Get the name of the user associated with the context
    let auth_info_name = &context_context.user;

    // Find the auth_info with the matching name
    let auth_info = kubeconfig
        .auth_infos
        .iter_mut()
        .find(|ai| ai.name == *auth_info_name)
        .ok_or("Failed to find auth_info in kubeconfig")?;

    // Unwrap the auth_info
    let auth_info_info = auth_info
        .auth_info
        .as_mut()
        .ok_or("AuthInfo is missing in NamedAuthInfo")?;

    // Remove the auth_provider and set the token
    auth_info_info.auth_provider = None;
    auth_info_info.token = Some(Secret::new(new_id_token));

    // Build the Kubernetes client with the updated kubeconfig
    let config = KubeConfig::from_custom_kubeconfig(kubeconfig, &Default::default()).await?;

    let client = Client::try_from(config)?;

    // Get pods from RCP
    let pods: Api<Pod> = Api::namespaced(client, &app_config.kube_namespace);
    let lp = ListParams::default();
    let pods: Vec<PodName> = pods
        .list(&lp)
        .await?
        .into_iter()
        .map(|pod| PodName::from(pod.metadata.name.clone().unwrap()))
        .collect();

    // If match_deployment is true, filter out pods that don't match the deployment
    if match_deployment {
        Ok(Some(
            pods.into_iter()
                .filter(|pod| pod.prefix == app_config.pod_prefix)
                .collect(),
        ))
    } else {
        Ok(Some(pods))
    }
}
