use super::db::ActiveModel;
use chrono::NaiveDateTime;
use sea_orm::{DeriveIntoActiveModel, NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Debug)]
pub struct Submission {
    id: Uuid,
    name: String,
    processing_has_started: bool,
    processing_success: bool,
    comment: Option<String>,
    created_on: NaiveDateTime,
    last_updated: NaiveDateTime,
    pub(super) associations: Vec<crate::uploads::models::UploadRead>,
    outputs: Vec<crate::external::s3::models::OutputObjectResponse>,
    // status: Vec<super::run_status::models::RunStatus>,
    status: Vec<crate::external::k8s::models::PodName>,
}

impl From<super::db::Model> for Submission {
    fn from(model: super::db::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            processing_has_started: model.processing_has_started,
            processing_success: model.processing_success,
            comment: model.comment,
            created_on: model.created_on,
            last_updated: model.last_updated,
            associations: vec![],
            outputs: vec![],
            status: vec![],
        }
    }
}

impl
    From<(
        super::db::Model,
        Vec<crate::uploads::db::Model>,
        // Vec<super::run_status::db::Model>,
        Vec<crate::external::k8s::models::PodName>,
        Vec<crate::external::s3::models::OutputObject>,
    )> for Submission
{
    fn from(
        model_tuple: (
            super::db::Model,
            Vec<crate::uploads::db::Model>,
            // Vec<super::run_status::db::Model>,
            Vec<crate::external::k8s::models::PodName>,
            Vec<crate::external::s3::models::OutputObject>,
        ),
    ) -> Self {
        let submission = model_tuple.0;
        let uploads = model_tuple.1;
        let status = model_tuple.2;
        let mut outputs: Vec<crate::external::s3::models::OutputObjectResponse> = model_tuple
            .3
            .into_iter()
            .map(|output| output.into())
            .collect();

        // Set the url for each output object
        for output in outputs.iter_mut() {
            output.url = Some(format!(
                "/api/submissions/{}/{}",
                submission.id, output.filename
            ));
        }
        Self {
            id: submission.id,
            name: submission.name,
            processing_has_started: submission.processing_has_started,
            processing_success: submission.processing_success,
            comment: submission.comment,
            created_on: submission.created_on,
            last_updated: submission.last_updated,
            associations: uploads
                .into_iter()
                .map(|association| association.into())
                .collect(),
            status: status.into_iter().map(|status| status.into()).collect(),
            outputs: outputs,
        }
    }
}

#[derive(ToSchema, Deserialize, Serialize, DeriveIntoActiveModel)]
pub struct SubmissionCreate {
    pub name: String,
    pub comment: Option<String>,
}

#[derive(ToSchema, Deserialize)]
pub struct SubmissionUpdate {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub comment: Option<Option<String>>,
}

impl From<SubmissionUpdate> for ActiveModel {
    fn from(update: SubmissionUpdate) -> Self {
        // If the field is Some(None), update the field to None, if None,
        // do not update the field (double option)

        Self {
            name: match update.name {
                Some(Some(name)) => Set(name),
                Some(_) => NotSet,
                _ => NotSet,
            },
            comment: match update.comment {
                Some(Some(comment)) => Set(Some(comment)),
                Some(_) => Set(None),
                _ => NotSet,
            },
            last_updated: Set(chrono::Utc::now().naive_utc()),
            id: NotSet,
            processing_has_started: NotSet,
            processing_success: NotSet,
            created_on: NotSet,
        }
    }
}
impl SubmissionUpdate {
    pub fn merge_into_activemodel(&self, mut model: ActiveModel) -> ActiveModel {
        // If the field is Some(None), update the field to None, if None,
        // do not update the field (double option)

        model.name = match self.name {
            Some(Some(ref name)) => Set(name.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.comment = match self.comment {
            Some(Some(ref comment)) => Set(Some(comment.clone())),
            Some(_) => Set(None),
            _ => NotSet,
        };
        model.last_updated = Set(chrono::Utc::now().naive_utc());

        model
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadPath {
    pub url: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Claims {
    pub(super) submission_id: Uuid,
    pub(super) filename: String,
    pub(super) exp: usize,
}
