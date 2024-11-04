// use crate::external::k8s::models::PodName;
// use crate::external::k8s::services::get_jobs_for_submission_id;
// use chrono::Utc;
// use sea_orm::*;
// use std::sync::Arc;
// use std::time::Duration;
// use tokio::time::sleep;
// use uuid::Uuid;

// pub async fn iteratively_check_status(
//     db: Arc<DatabaseConnection>,
//     s3_client: Arc<S3Client>,
//     submission_id: Uuid,
//     job_id: String,
//     timeout: Duration,
// ) -> Result<(), sea_orm::DbErr> {
//     println!(
//         "Spooling up submission status check for {} - {}",
//         submission_id, job_id
//     );

//     let mut submission_jobs: Vec<PodName> = match get_jobs_for_submission_id(submission_id).await {
//         Ok(jobs) => jobs.into_iter().find(|j| j.submission_id == submission_id),
//         Err(e) => {
//             println!(
//                 "Error fetching job {} for submission {}: {}",
//                 job_id, submission_id, e
//             );
//             vec![]
//         }
//     };

//     let time_started = Utc::now();

//     while job.is_none() {
//         println!("Job {} is not ready yet", job_id);
//         sleep(Duration::from_secs(5)).await;

//         job = get_submission_job_by_name(&db, submission_id, &job_id).await?;
//         if Utc::now().signed_duration_since(time_started).num_seconds() > timeout.as_secs() as i64 {
//             println!(
//                 "Submission {} has timed out waiting for update. Cancelling job {}",
//                 submission_id, job_id
//             );
//             return Ok(());
//         }
//     }

//     // Initialize run status in the database
//     let run_status = RunStatus {
//         kubernetes_pod_name: Some(job_id.clone()),
//         submission_id,
//         status: Some("Pending".to_string()),
//         is_running: true,
//         is_successful: false,
//         ..Default::default()
//     };
//     run_status.insert(&db).await?;

//     // Polling loop to check job status
//     while let Some(ref mut job) = job {
//         let logs = get_cached_job_log(&format!("{}-0-0", job_id)).await?;

//         let update_query = RunStatus::update()
//             .filter(run_status::Column::KubernetesPodName.eq(job_id.clone()))
//             .set(run_status::Column::Status, job.status.clone())
//             .set(run_status::Column::IsRunning, true)
//             .set(
//                 run_status::Column::TimeStarted,
//                 Some(job.time_started.clone()),
//             )
//             .set(
//                 run_status::Column::LastUpdated,
//                 Some(Utc::now().naive_utc()),
//             )
//             .set(run_status::Column::Logs, logs);

//         db.execute(update_query).await?;

//         println!(
//             "Submission ID: {}: Job {} is running",
//             submission_id, job_id
//         );

//         // Sleep and check the job status again
//         sleep(Duration::from_secs(5)).await;
//         job = get_submission_job_by_name(&db, submission_id, &job_id).await?;
//     }

//     println!("Loop finished");

//     // Final update once job completes or fails
//     let final_logs = get_cached_job_log(&format!("{}-0-0", job_id)).await?;
//     let final_status = job
//         .as_ref()
//         .map(|j| j.status.as_deref() == Some("Succeeded"))
//         .unwrap_or(false);
//     let is_k8s_resource = job.is_some();

//     let final_update = RunStatus::update()
//         .filter(run_status::Column::KubernetesPodName.eq(job_id.clone()))
//         .set(run_status::Column::IsRunning, false)
//         .set(run_status::Column::IsSuccessful, final_status)
//         .set(
//             run_status::Column::IsStillKubernetesResource,
//             is_k8s_resource,
//         )
//         .set(
//             run_status::Column::LastUpdated,
//             Some(Utc::now().naive_utc()),
//         )
//         .set(
//             run_status::Column::Status,
//             job.as_ref()
//                 .and_then(|j| j.status.clone())
//                 .unwrap_or_else(|| "Deleted".to_string()),
//         )
//         .set(run_status::Column::Logs, final_logs);

//     db.execute(final_update).await?;
//     println!("Submission ID: {} has finished running.", submission_id);

//     Ok(())
// }

use crate::uploads::db;
use anyhow::{anyhow, Error, Result};
use sea_orm::{DatabaseConnection, ModelTrait};

pub(super) async fn get_input_objects(
    submission_obj: super::db::Model,
    db: &DatabaseConnection,
) -> Result<Vec<db::Model>, Error> {
    // Get the related objects to the submission according to the association
    match submission_obj
        .find_related(crate::uploads::db::Entity)
        .all(db)
        .await
    {
        // Return all or none. If any fail, return an error
        Ok(uploads) => Ok(uploads),
        Err(_) => Err(anyhow!("Failed to fetch uploads")),
    }
}
