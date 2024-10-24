use super::models::{ChangeFileInfo, PreCreateResponse};
use crate::config::Config;
use crate::external::tus::models::EventPayload;
use crate::uploads::associations::db as AssociationDB;
use crate::uploads::db as InputObjectDB;
use anyhow::Result;
use chrono::Utc;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

pub(super) async fn handle_pre_create(
    db: DatabaseConnection,
    payload: EventPayload,
) -> Result<PreCreateResponse> {
    println!("Handling pre-create");
    let config: Config = Config::from_env();
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

    let allowed_types: Vec<&str> = vec!["application/octet-stream"];
    let allowed_file_extensions: Vec<&str> = vec!["pod5"];

    if !allowed_types.contains(&filetype.clone().as_str()) {
        return Err(anyhow::anyhow!("Filetype not allowed"));
    }
    if !allowed_file_extensions.contains(&filename.split('.').last().unwrap()) {
        return Err(anyhow::anyhow!("File extension not allowed"));
    }

    let object = InputObjectDB::ActiveModel {
        id: Set(Uuid::new_v4()),
        created_on: Set(Utc::now().naive_utc()),
        filename: Set(filename.clone()),
        size_bytes: Set(size_in_bytes),
        all_parts_received: Set(false),
        last_part_received: Set(Some(Utc::now().naive_utc())),
        processing_message: Set(Some("Upload initiated".to_string())),
        ..Default::default() // Assuming other fields use default
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
    println!("Creating s3 key");
    let s3_key = format!("{}/{}", config.s3_prefix, object.last_insert_id);
    println!("S3 key: {}", s3_key);

    match AssociationDB::Entity::insert(association_object)
        .exec(&db)
        .await
    {
        Ok(_) => (),
        _ => return Err(anyhow::anyhow!("Failed to create association")),
    }

    // Respond with a custom ID for tusd to upload to S3
    Ok(PreCreateResponse {
        change_file_info: Some(ChangeFileInfo { id: s3_key }),
        status: "success".to_string(),
    })
}

pub(super) async fn handle_post_create(
    db: DatabaseConnection,
    payload: EventPayload,
) -> Result<PreCreateResponse> {
    println!("Handling post-create");
    let upload_id = &payload.event.upload.id;
    let object_id: Uuid = upload_id
        .split('/')
        .nth(1)
        .unwrap()
        .split('+')
        .next()
        .unwrap()
        .try_into()
        .unwrap();

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
    // Split the s3_prefix and then the UUID out of the upload_id to get the object ID.
    // Then again with the + separator between UUID and TUSd upload ID
    let object_id: Uuid = upload_id
        .split('/')
        .nth(1)
        .unwrap()
        .split('+')
        .next()
        .unwrap()
        .try_into()
        .unwrap();

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
    let object_id: Uuid = upload_id
        .split('/')
        .nth(1)
        .unwrap()
        .split('+')
        .next()
        .unwrap()
        .try_into()
        .unwrap();

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

    // Unwrap the UUID from the upload_id. Something like this:
    // labcaller/<uuid>+<tusd_upload_id> -> <uuid>
    let object_id: Uuid = upload_id
        .split('/')
        .nth(1)
        .unwrap()
        .split('+')
        .next()
        .unwrap()
        .try_into()
        .unwrap();

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
        }),
        _ => Err(anyhow::anyhow!("Failed to update after upload completed")),
    }
}
