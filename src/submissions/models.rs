use super::db::ActiveModel;
use chrono::NaiveDateTime;
use sea_orm::{DeriveIntoActiveModel, FromQueryResult, NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, FromQueryResult)]
pub struct Submission {
    id: Uuid,
    name: String,
    processing_has_started: bool,
    processing_success: bool,
    comment: Option<String>,
    created_on: NaiveDateTime,
    last_updated: NaiveDateTime,
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
                Some(None) => NotSet,
                None => NotSet,
            },
            comment: match update.comment {
                Some(Some(comment)) => Set(Some(comment)),
                Some(None) => Set(None),
                None => NotSet,
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
            Some(None) => NotSet,
            None => NotSet,
        };

        model.comment = match self.comment {
            Some(Some(ref comment)) => Set(Some(comment.clone())),
            Some(None) => Set(None),
            None => NotSet,
        };
        model.last_updated = Set(chrono::Utc::now().naive_utc());

        model
    }
}
