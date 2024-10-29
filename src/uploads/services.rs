use super::associations;
use super::db;
use crate::config::Config;
use anyhow::Error;
use aws_sdk_s3::Client as S3Client;
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::sync::Arc;
use uuid::Uuid;

pub async fn delete_object(
    db: &DatabaseConnection,
    s3: &Arc<S3Client>,
    id: Uuid,
) -> Result<(), Error> {
    // Delete all associations for the object
    associations::db::Entity::delete_many()
        .filter(associations::db::Column::InputObjectId.eq(id.clone()))
        .exec(db)
        .await
        .map_err(|e| Error::new(e))?;

    // Find the object to delete
    match db::Entity::find_by_id(id).one(db).await? {
        Some(obj) => {
            let obj: db::ActiveModel = obj.into();

            // Delete from the database
            let res = db::Entity::delete(obj).exec(db).await?;
            if res.rows_affected == 0 {
                return Err(anyhow::anyhow!("Object not found"));
            }

            // Delete from S3
            let config = Config::from_env();
            s3.delete_object()
                .bucket(config.s3_bucket)
                .key(format!("{}/{}", config.s3_prefix, id))
                .send()
                .await
                .map_err(|e| Error::new(e))?;

            Ok(())
        }
        _ => Err(anyhow::anyhow!("Object not found")),
    }
}
