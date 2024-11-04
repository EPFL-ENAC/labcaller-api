use super::models::PodName;
use crate::config::Config;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams},
    config::Kubeconfig,
    Client, Config as KubeConfig,
};
use secrecy::Secret;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use uuid::Uuid;

#[derive(Deserialize)]
struct TokenResponse {
    id_token: String,
}

async fn refresh_oidc_token(refresh_token: &str, idp_issuer_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", "runai-cli"),
    ];
    let url = format!("{}/protocol/openid-connect/token", idp_issuer_url);
    let res = match client.post(&url).form(&params).send().await {
        Ok(res) => res,
        Err(e) => {
            return Err(anyhow!("Failed to refresh token: {}", e));
        }
    };

    if res.status().is_success() {
        let token_response: TokenResponse = res.json().await?;
        Ok(token_response.id_token)
    } else {
        let error_text = res.text().await?;
        Err(anyhow!("Failed to refresh token: {}", error_text))
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
    None
}

pub async fn get_pods() -> Result<Vec<crate::external::k8s::models::PodName>, kube::Error> {
    // Get app config and kube client
    let app_config = Config::from_env();
    let client = refresh_token_and_get_client().await.unwrap();

    // Get pods from Kubernetes API
    let pods: Api<Pod> = Api::namespaced(client, &app_config.kube_namespace);
    let lp = ListParams::default();

    let pod_list = match pods.list(&lp).await {
        Ok(pod_list) => pod_list,
        Err(e) => return Err(e),
    };
    let pod_infos: Vec<crate::external::k8s::models::PodName> = pod_list
        .clone()
        .items
        .into_iter()
        .filter_map(|pod| {
            let name = pod.metadata.name.clone()?;

            // Add this line to filter by prefix
            if !name.starts_with(&app_config.pod_prefix) {
                return None;
            }

            let start_time: Option<DateTime<Utc>> = match pod.status.clone().unwrap().start_time {
                Some(time) => Some(time.0),
                None => None,
            };

            let phase = pod.status.as_ref().and_then(|status| status.phase.clone());

            // Get the latest status time by the latest container status.conditions ordered by last_transition_time
            let latest_status_time: Option<DateTime<Utc>> =
                match pod.status.as_ref().and_then(|status| {
                    status.conditions.as_ref().and_then(|conditions| {
                        conditions
                            .iter()
                            .max_by_key(|condition| condition.last_transition_time.clone())
                            .map(|condition| condition.last_transition_time.clone())
                    })
                }) {
                    Some(time) => Some(time.unwrap().0),
                    None => None,
                };

            Some(
                crate::external::k8s::models::PodInfo {
                    name,
                    start_time,
                    latest_status: phase.unwrap_or_else(|| "Unknown".to_string()),
                    latest_status_time,
                }
                .into(),
            )
        })
        .collect();

    Ok(pod_infos)
}

pub async fn refresh_token_and_get_client() -> Result<Client> {
    let app_config = Config::from_env();

    // Read and parse the kubeconfig file
    let mut kubeconfig = {
        let mut file = File::open(&app_config._kube_config)?;
        let mut yaml_str = String::new();
        file.read_to_string(&mut yaml_str)?;
        serde_yaml::from_str::<Kubeconfig>(&yaml_str)?
    };

    let refresh_token = extract_refresh_token(&kubeconfig)
        .ok_or_else(|| anyhow!("Failed to find refresh token in kubeconfig"))?;

    // Get the idp-issuer-url from the kubeconfig for the refresh token
    let idp_issuer_url = kubeconfig.auth_infos[0]
        .auth_info
        .as_ref()
        .unwrap()
        .auth_provider
        .as_ref()
        .unwrap()
        .config
        .get("idp-issuer-url")
        .unwrap();

    // Refresh the OIDC token
    let new_id_token = refresh_oidc_token(&refresh_token, &idp_issuer_url).await?;

    // Update the kubeconfig's auth_info
    // Find the current context name
    let current_context_name = kubeconfig
        .current_context
        .clone()
        .ok_or_else(|| anyhow!("No current context set in kubeconfig"))?;

    // Find the context that matches the current context name
    let context = kubeconfig
        .contexts
        .iter()
        .find(|ctx| ctx.name == current_context_name)
        .ok_or_else(|| anyhow!("Failed to find current context in kubeconfig"))?;

    // Unwrap the context
    let context_context = context
        .context
        .as_ref()
        .ok_or_else(|| anyhow!("Context is missing in NamedContext"))?;

    // Get the name of the user associated with the context
    let auth_info_name = &context_context.user;

    // Find the auth_info with the matching name
    let auth_info = kubeconfig
        .auth_infos
        .iter_mut()
        .find(|ai| ai.name == *auth_info_name)
        .ok_or_else(|| anyhow!("Failed to find auth_info in kubeconfig"))?;

    // Unwrap the auth_info
    let auth_info_info = auth_info
        .auth_info
        .as_mut()
        .ok_or_else(|| anyhow!("AuthInfo is missing in NamedAuthInfo"))?;

    // Remove the auth_provider and set the token
    auth_info_info.auth_provider = None;
    auth_info_info.token = Some(Secret::new(new_id_token));

    // Build the Kubernetes client with the updated kubeconfig
    let config = KubeConfig::from_custom_kubeconfig(kubeconfig, &Default::default()).await?;

    let client = Client::try_from(config)?;
    Ok(client)
}

pub async fn get_jobs_for_submission_id(submission_id: Uuid) -> Result<Vec<PodName>> {
    // Get app config and kube client
    let pods = get_pods().await.unwrap();

    // Filter pods by submission_id
    let jobs: Vec<PodName> = pods
        .into_iter()
        .filter(|pod| pod.submission_id == submission_id)
        .collect();

    Ok(jobs)
}
