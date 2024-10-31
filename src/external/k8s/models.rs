use crate::{config::Config, submissions};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use thiserror::Error;
use tokio::io::split;
use uuid::Uuid;

#[derive(Debug)]
pub struct PodInfo {
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub latest_status: String,
    pub latest_status_time: DateTime<Utc>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PodName {
    pub prefix: String,
    pub submission_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub latest_status: String,
    pub latest_status_time: DateTime<Utc>,
    pub run_id: u64,
}

#[derive(Error, Debug)]
pub enum PodNameError {
    #[error("Pod name does not have the expected structure")]
    InvalidStructure,
    #[error("Pod name does not have the expected prefix")]
    InvalidPrefix,
    #[error("Invalid UUID format")]
    InvalidUuid(#[from] uuid::Error),
    #[error("Invalid run ID")]
    InvalidRunId(#[from] std::num::ParseIntError),
}

impl From<PodInfo> for PodName {
    fn from(pod_info: PodInfo) -> Self {
        let config = Config::from_env();

        // Strip the prefix from the pod name, regardless of hyphens
        let name_without_prefix = pod_info
            .name
            .strip_prefix(&format!("{}-", config.pod_prefix))
            .unwrap_or(&pod_info.name); // fallback if prefix is absent

        // Reverse split to isolate <UUID>-<run_id>-x-x parts
        let parts: Vec<&str> = name_without_prefix.rsplitn(4, '-').collect();

        if parts.len() < 4 {
            panic!("Pod name does not have the expected structure");
        }

        let run_id: u64 = parts[2].parse().expect("Invalid run ID format");
        let submission_id = Uuid::parse_str(parts[3]).expect("Invalid UUID format");

        PodName {
            prefix: config.pod_prefix.clone(),
            submission_id,
            start_time: pod_info.start_time,
            latest_status: pod_info.latest_status,
            latest_status_time: pod_info.latest_status_time,
            run_id,
        }
    }
}
