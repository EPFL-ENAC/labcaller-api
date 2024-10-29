use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[kube(
    group = "run.ai",
    version = "v2alpha1",
    kind = "TrainingWorkload",
    namespaced
)]
pub struct TrainingWorkloadSpec {
    pub allow_privilege_escalation: Option<ValueField<bool>>,
    pub environment: Environment,
    pub gpu: ValueField<String>, // Using ValueField to match `value` structure
    pub image: ValueField<String>,
    #[serde(rename = "imagePullPolicy")]
    pub image_pull_policy: ValueField<String>,
    pub name: ValueField<String>,
    pub run_as_gid: Option<ValueField<u32>>,
    pub run_as_uid: Option<ValueField<u32>>,
    pub run_as_user: Option<ValueField<bool>>,
    pub service_type: Option<ValueField<String>>,
    pub usage: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Environment {
    pub items: EnvironmentItems,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EnvironmentItems {
    pub input_object_ids: ValueField<String>,
    pub s3_access_key: ValueField<String>,
    pub s3_bucket_id: ValueField<String>,
    pub s3_prefix: ValueField<String>,
    pub s3_secret_key: ValueField<String>,
    pub s3_url: ValueField<String>,
    pub submission_id: ValueField<String>,
    pub base_image: ValueField<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct ValueField<T> {
    pub value: T,
}
