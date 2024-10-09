use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Debug, Clone)]
pub struct PodName {
    pub prefix: String,
    pub submission_id: Uuid,
    pub run_id: u64,
}

impl From<String> for PodName {
    fn from(pod_name: String) -> Self {
        let parts: Vec<&str> = pod_name.split('-').collect();
        if parts.len() < 7 {
            println!("Pod name does not have the expected structure");
        }
        let uuid_str = format!(
            "{}-{}-{}-{}-{}",
            parts[1], parts[2], parts[3], parts[4], parts[5]
        );
        let submission_id = Uuid::parse_str(&uuid_str).unwrap();
        let run_id: u64 = parts[6].parse().unwrap();
        let prefix: String = parts[0].to_string();

        PodName {
            prefix,
            submission_id,
            run_id,
        }
    }
}
