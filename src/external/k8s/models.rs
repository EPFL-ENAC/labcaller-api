use crate::config::Config;
use anyhow::Result;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Serialize, Debug, Clone)]
pub struct PodName {
    pub prefix: String,
    pub submission_id: Uuid,
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

impl TryFrom<String> for PodName {
    type Error = PodNameError;

    fn try_from(pod_name: String) -> Result<Self, Self::Error> {
        let app_config = Config::from_env();
        let parts: Vec<&str> = pod_name.split('-').collect();

        // Check that the pod name has the expected structure and prefix
        if parts.len() < 7 {
            return Err(PodNameError::InvalidStructure);
        }

        if parts[0] != app_config.pod_prefix {
            return Err(PodNameError::InvalidPrefix);
        }

        let uuid_str = format!(
            "{}-{}-{}-{}-{}",
            parts[1], parts[2], parts[3], parts[4], parts[5]
        );

        let submission_id = Uuid::parse_str(&uuid_str)?;
        let run_id: u64 = parts[6].parse()?;

        Ok(PodName {
            prefix: parts[0].to_string(),
            submission_id,
            run_id,
        })
    }
}
