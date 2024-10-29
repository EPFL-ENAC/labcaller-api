use super::models::{ChangeFileInfo, PreCreateResponse};
use crate::external::tus::models::{EventPayload, HttpResponse};
use crate::submissions::db as SubmissionDB;
use crate::uploads::associations::db as AssociationDB;
use crate::uploads::db as InputObjectDB;
use anyhow::Result;
use chrono::Utc;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, Set};
use uuid::Uuid;

pub(super) async fn handle_pre_create(
    db: DatabaseConnection,
    payload: EventPayload,
) -> Result<PreCreateResponse> {
    println!("Handling pre-create");

    let filename = payload.event.upload.metadata.filename;
    let filetype = payload.event.upload.metadata.filetype;
    let size_in_bytes = payload.event.upload.size;
    let submission_id: Uuid = match payload.event.http_request.header.submission_id {
        Some(submission_id) => match submission_id.get(0).unwrap().parse() {
            Ok(submission_id) => submission_id,
            _ => Err(anyhow::anyhow!("Failed to parse submission ID"))?,
        },
        _ => Err(anyhow::anyhow!("Submission ID not found"))?,
    };
    println!("Submission ID: {}", submission_id);
    println!("Filename: {}, Filetype: {}", filename, filetype);

    // Check that the submission does not already have that same filename
    // let existing_file = InputObjectDB::Entity::find()
    //     .filter(InputObjectDB::Column::Filename.eq(filename.clone()))
    //     .find_also_related(AssociationDB::Entity)
    //     .filter(AssociationDB::Column::SubmissionId.eq(submission_id))
    //     .one(&db)
    //     .await?;

    let results: Vec<(SubmissionDB::Model, Vec<InputObjectDB::Model>)> =
        SubmissionDB::Entity::find()
            .filter(SubmissionDB::Column::Id.eq(submission_id))
            .find_with_related(InputObjectDB::Entity)
            .filter(InputObjectDB::Column::Filename.eq(filename.clone()))
            .all(&db)
            .await
            .unwrap();

    // Unpack the tuples, if there are any input objects, then the filename is already in use
    if results.iter().any(|(_, objs)| objs.len() > 0) {
        return Ok(PreCreateResponse {
            // change_file_info: Some(ChangeFileInfo {
            // id: object.last_insert_id.to_string(),
            // }),
            status: "success".to_string(),
            http_response: Some(HttpResponse {
                status_code: Some(400),
                body: Some(
                    "File already uploaded with this filename in this submission".to_string(),
                ),
                ..Default::default()
            }),
            reject_upload: true,
            ..Default::default()
        });
    }

    let allowed_types: Vec<&str> = vec!["application/octet-stream"];
    let allowed_file_extensions: Vec<&str> = vec!["pod5"];

    if !allowed_types.contains(&filetype.clone().as_str()) {
        return Err(anyhow::anyhow!("Filetype not allowed"));
    }
    if !allowed_file_extensions.contains(&filename.split('.').last().unwrap()) {
        return Err(anyhow::anyhow!("File extension not allowed"));
    }

    // Check that the submission does not already have that same filename
    let object = InputObjectDB::ActiveModel {
        id: Set(Uuid::new_v4()),
        created_on: Set(Utc::now().naive_utc()),
        filename: Set(filename.clone()),
        size_bytes: Set(size_in_bytes),
        all_parts_received: Set(false),
        last_part_received: Set(Some(Utc::now().naive_utc())),
        processing_message: Set(Some("Upload initiated".to_string())),
        ..Default::default()
    };

    let object = match InputObjectDB::Entity::insert(object).exec(&db).await {
        Ok(obj) => obj,
        _ => return Err(anyhow::anyhow!("Failed to create object")),
    };

    let association_object = AssociationDB::ActiveModel {
        input_object_id: Set(object.last_insert_id),
        submission_id: Set(submission_id),
        ..Default::default()
    };

    match AssociationDB::Entity::insert(association_object)
        .exec(&db)
        .await
    {
        Ok(_) => (),
        _ => return Err(anyhow::anyhow!("Failed to create association")),
    }

    // Respond with a custom ID for tusd to upload to S3
    Ok(PreCreateResponse {
        change_file_info: Some(ChangeFileInfo {
            id: object.last_insert_id.to_string(),
        }),
        status: "success".to_string(),
        ..Default::default()
    })
}

pub(super) async fn handle_post_create(
    db: DatabaseConnection,
    payload: EventPayload,
) -> Result<PreCreateResponse> {
    println!("Handling post-create");
    let upload_id = &payload.event.upload.id;
    // Split the upload_id on the + separator to get the object ID.
    let object_id: Uuid = match upload_id
        .split('+')
        .next()
        .and_then(|id_str| Uuid::parse_str(id_str).ok())
    {
        Some(id) => id,
        _ => {
            println!("Invalid object ID in upload_id");
            return Err(anyhow::anyhow!("Invalid object ID in upload_id"));
        }
    };

    let obj = match InputObjectDB::Entity::find()
        .filter(InputObjectDB::Column::Id.eq(object_id))
        .one(&db)
        .await
    {
        Ok(obj) => obj,
        _ => return Err(anyhow::anyhow!("Failed to find object")),
    };

    let mut obj: InputObjectDB::ActiveModel = obj.unwrap().into();

    obj.processing_message = Set(Some(format!("Upload started").to_owned()));
    obj.last_part_received = Set(Some(Utc::now().naive_utc().to_owned()));

    // let obj: db::Model = db::Entity::update(obj).exec(&db).await.unwrap();
    match InputObjectDB::Entity::update(obj).exec(&db).await {
        Ok(_) => Ok(PreCreateResponse {
            change_file_info: None,
            status: "Upload accepted".to_string(),
            ..Default::default()
        }),
        _ => Err(anyhow::anyhow!("Failed to update after upload started")),
    }
}

pub(super) async fn handle_post_receive(
    db: DatabaseConnection,
    payload: EventPayload,
) -> Result<PreCreateResponse> {
    println!("Handling post-receive");
    let upload_id = &payload.event.upload.id;
    // Split the upload_id on the + separator to get the object ID.
    let object_id: Uuid = match upload_id
        .split('+')
        .next()
        .and_then(|id_str| Uuid::parse_str(id_str).ok())
    {
        Some(id) => id,
        _ => {
            println!("Invalid object ID in upload_id");
            return Err(anyhow::anyhow!("Invalid object ID in upload_id"));
        }
    };

    let size_in_bytes = payload.event.upload.size;
    let offset = payload.event.upload.offset;
    let uploaded_percentage = (offset as f64 / size_in_bytes as f64) * 100.0;

    let obj = match InputObjectDB::Entity::find()
        .filter(InputObjectDB::Column::Id.eq(object_id))
        .one(&db)
        .await
    {
        Ok(obj) => obj,
        _ => return Err(anyhow::anyhow!("Failed to find object")),
    };

    // Don't update if all parts have been received, it's already 100%
    if obj.clone().unwrap().all_parts_received == true {
        return Ok(PreCreateResponse {
            change_file_info: None,
            status: "Upload progress updated".to_string(),
            ..Default::default()
        });
    }
    let mut obj: InputObjectDB::ActiveModel = obj.unwrap().into();

    obj.processing_message = Set(Some(
        format!("Upload progress: {:.2}%", uploaded_percentage).to_owned(),
    ));
    obj.last_part_received = Set(Some(Utc::now().naive_utc().to_owned()));

    // let obj: db::Model = db::Entity::update(obj).exec(&db).await.unwrap();
    match InputObjectDB::Entity::update(obj).exec(&db).await {
        Ok(_) => Ok(PreCreateResponse {
            change_file_info: None,
            status: "Upload progress updated".to_string(),
            ..Default::default()
        }),
        _ => Err(anyhow::anyhow!("Failed to update upload progress")),
    }
}

pub(super) async fn handle_pre_finish(
    db: DatabaseConnection,
    payload: EventPayload,
) -> Result<PreCreateResponse> {
    println!("Handling pre-finish");
    let upload_id = &payload.event.upload.id;
    // Split the upload_id on the + separator to get the object ID.
    let object_id: Uuid = match upload_id
        .split('+')
        .next()
        .and_then(|id_str| Uuid::parse_str(id_str).ok())
    {
        Some(id) => id,
        _ => {
            println!("Invalid object ID in upload_id");
            return Err(anyhow::anyhow!("Invalid object ID in upload_id"));
        }
    };

    let obj = match InputObjectDB::Entity::find()
        .filter(InputObjectDB::Column::Id.eq(object_id))
        .one(&db)
        .await
    {
        Ok(obj) => obj,
        _ => return Err(anyhow::anyhow!("Failed to find object")),
    };

    let mut obj: InputObjectDB::ActiveModel = obj.unwrap().into();

    obj.processing_message = Set(Some("Upload completed".to_owned()));
    obj.all_parts_received = Set(true);
    obj.last_part_received = Set(Some(Utc::now().naive_utc().to_owned()));

    match InputObjectDB::Entity::update(obj).exec(&db).await {
        Ok(_) => Ok(PreCreateResponse {
            change_file_info: None,
            status: "Upload completed".to_string(),
            ..Default::default()
        }),
        _ => Err(anyhow::anyhow!("Failed to update after upload completed")),
    }
}

pub(super) async fn handle_post_finish(
    db: DatabaseConnection,
    payload: EventPayload,
) -> Result<PreCreateResponse> {
    println!("Handling post-finish");
    let upload_id = &payload.event.upload.id;

    // Split the upload_id on the + separator to get the object ID.
    let object_id: Uuid = match upload_id
        .split('+')
        .next()
        .and_then(|id_str| Uuid::parse_str(id_str).ok())
    {
        Some(id) => id,
        _ => {
            println!("Invalid object ID in upload_id");
            return Err(anyhow::anyhow!("Invalid object ID in upload_id"));
        }
    };

    let obj = match InputObjectDB::Entity::find()
        .filter(InputObjectDB::Column::Id.eq(object_id))
        .one(&db)
        .await
    {
        Ok(obj) => obj,
        _ => return Err(anyhow::anyhow!("Failed to find object")),
    };

    let mut obj: InputObjectDB::ActiveModel = obj.unwrap().into();

    obj.processing_message = Set(Some("Upload completed".to_owned()));
    obj.all_parts_received = Set(true);
    obj.last_part_received = Set(Some(Utc::now().naive_utc().to_owned()));

    match InputObjectDB::Entity::update(obj).exec(&db).await {
        Ok(_) => Ok(PreCreateResponse {
            change_file_info: None,
            status: "Upload completed".to_string(),
            ..Default::default()
        }),
        _ => Err(anyhow::anyhow!("Failed to update after upload completed")),
    }
}

pub(super) async fn handle_post_terminate(
    db: DatabaseConnection,
    payload: EventPayload,
) -> Result<PreCreateResponse> {
    // This hook is sent when the file should be cleaned up (del from db)
    println!("Handling post-terminate");
    let upload_id = &payload.event.upload.id;

    // Split the upload_id on the + separator to get the object ID.
    let object_id: Uuid = match upload_id
        .split('+')
        .next()
        .and_then(|id_str| Uuid::parse_str(id_str).ok())
    {
        Some(id) => id,
        _ => {
            println!("Invalid object ID in upload_id");
            return Err(anyhow::anyhow!("Invalid object ID in upload_id"));
        }
    };

    let obj = match InputObjectDB::Entity::find()
        .filter(InputObjectDB::Column::Id.eq(object_id))
        .one(&db)
        .await
    {
        Ok(obj) => obj,
        _ => return Err(anyhow::anyhow!("Failed to find object")),
    };

    // Delete all associations, then delete the object
    AssociationDB::Entity::delete_many()
        .filter(AssociationDB::Column::InputObjectId.eq(object_id))
        .exec(&db)
        .await
        .unwrap();

    let obj = obj.unwrap();
    match obj.delete(&db).await {
        Ok(_) => Ok(PreCreateResponse {
            change_file_info: None,
            status: "Upload terminated".to_string(),
            ..Default::default()
        }),
        _ => Err(anyhow::anyhow!("Failed to delete object")),
    }
}
