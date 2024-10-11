use crate::common::auth::Role;
use crate::common::filter::{apply_filters, parse_range};
use crate::common::models::FilterOptions;
use crate::common::pagination::calculate_content_range;
use crate::common::sort::generic_sort;
use crate::external::s3::services::upload_stream;
use aws_config::BehaviorVersion;
// use aws_sdk_s3;
// use aws_sdk_s3::operation::create_multipart_upload::CreateMultipartUploadOutput;
// use aws_sdk_s3::;
use aws_sdk_s3::{
    config::Credentials,
    config::Region,
    operation::create_multipart_upload::CreateMultipartUploadOutput,
    types::{CompletedMultipartUpload, CompletedPart},
    Client as S3Client,
};
use aws_smithy_types::byte_stream::ByteStream;
use axum::{
    debug_handler,
    extract::{DefaultBodyLimit, Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use axum_keycloak_auth::{
    instance::KeycloakAuthInstance, layer::KeycloakAuthLayer, PassthroughMode,
};
use futures::stream::StreamExt;
use sea_orm::{
    query::*, ActiveModelTrait, DatabaseConnection, DeleteResult, EntityTrait, IntoActiveModel,
    ModelTrait, Set, SqlErr,
};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
// use tokio_util::io::StreamReader;
use tokio::sync::Semaphore;
use uuid::Uuid;

const RESOURCE_NAME: &str = "uploads";
const PART_SIZE: usize = 5 * 1024 * 1024; // Minimum part size for S3 multipart upload
const MAX_CONCURRENT_UPLOADS: usize = 10; // Limit the number of concurrent uploads

pub fn router(db: DatabaseConnection, keycloak_auth_instance: Arc<KeycloakAuthInstance>) -> Router {
    Router::new()
        .route("/", routing::post(upload_one).get(get_all))
        .with_state(db)
        .layer(DefaultBodyLimit::max(1073741824))
    // .layer(
    //     KeycloakAuthLayer::<Role>::builder()
    //         .instance(keycloak_auth_instance)
    //         .passthrough_mode(PassthroughMode::Block)
    //         .persist_raw_claims(false)
    //         .expected_audiences(vec![String::from("account")])
    //         .required_roles(vec![Role::Administrator])
    //         .build(),
    // )
}

#[axum::debug_handler]
pub async fn upload_one(State(db): State<DatabaseConnection>, mut multipart: Multipart) {
    let app_conf = crate::config::Config::from_env();
    println!("Accessed route");
    let region = Region::new("us-east-1");
    let credentials = Credentials::new(
        &app_conf.s3_access_key,
        &app_conf.s3_secret_key,
        None,
        None,
        "manual",
    );
    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region.clone())
        .credentials_provider(credentials)
        .endpoint_url(&app_conf.s3_url)
        .load()
        .await;

    let client = Arc::new(S3Client::new(&shared_config));

    // Add the prefix to the key

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let filename = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let key = format!("{}/{}", app_conf.s3_prefix, filename);

        // Start the multipart upload
        let multipart_upload_res: CreateMultipartUploadOutput = client
            .create_multipart_upload()
            .bucket(&app_conf.s3_bucket)
            .key(&key)
            .send()
            .await
            .expect("Couldn't create multipart upload");

        let upload_id = multipart_upload_res.upload_id().unwrap().to_string();

        println!(
            "Length of `{}: {filename}` is {} megabytes. Upload ID: {}",
            name,
            data.len() / 1024 / 1024,
            upload_id,
        );

        // Get amount of uploaded parts from the upload_id from S3
        let parts = client
            .list_parts()
            .bucket(&app_conf.s3_bucket)
            .key(&key)
            .upload_id(&upload_id)
            .send()
            .await
            .expect("Couldn't list parts");

        println!("Parts: {:?}", parts);
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_UPLOADS)); // Limit concurrency
        let mut upload_futures = Vec::new();
        let mut part_number = 1;
        let mut eof = false;
        let mut stream = tokio::io::BufReader::new(data.as_ref());

        // If the file is larger than 5MB, parallel upload in parts
        if data.len() > PART_SIZE {
            while !eof {
                let client = Arc::clone(&client);
                let key = key.clone();
                let upload_id = upload_id.clone();
                let bucket = app_conf.s3_bucket.clone();
                let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap(); // Acquire semaphore permit

                let mut buffer = vec![0u8; PART_SIZE];
                let mut bytes_read = 0;

                // Read data into buffer
                while bytes_read < PART_SIZE {
                    match stream.read(&mut buffer[bytes_read..]).await {
                        Ok(0) => {
                            eof = true;
                            break;
                        }
                        Ok(n) => {
                            bytes_read += n;
                        }
                        Err(e) => {
                            eprintln!("Error reading stream: {}", e);
                            return;
                        }
                    }
                }

                if bytes_read == 0 {
                    break; // No more data to read
                }

                let data = buffer[..bytes_read].to_vec();

                // Upload each part concurrently using tokio::spawn
                let upload_future = tokio::spawn(async move {
                    let part = client
                        .upload_part()
                        .key(&key)
                        .bucket(&bucket)
                        .upload_id(&upload_id)
                        .body(ByteStream::from(data))
                        .part_number(part_number)
                        .send()
                        .await
                        .expect("Couldn't upload part");

                    drop(permit); // Release semaphore permit when the upload is done

                    CompletedPart::builder()
                        .e_tag(part.e_tag().unwrap_or_default())
                        .part_number(part_number)
                        .build()
                });

                upload_futures.push(upload_future);

                // Move to the next part
                part_number += 1;
            }
            // futures::future::join_all(upload_futures).await;

            let completed_parts = futures::future::join_all(upload_futures)
                .await
                .into_iter()
                .map(|result| result.unwrap())
                .collect::<Vec<CompletedPart>>();

            let completed_multipart_upload: CompletedMultipartUpload =
                CompletedMultipartUpload::builder()
                    .set_parts(Some(completed_parts.clone()))
                    .build();

            println!("Completing multipart upload in {} parts", part_number);
            println!("Completed parts: {:?}", completed_parts);
            println!(
                "Completed multipart upload: {:?}",
                completed_multipart_upload
            );
        } else {
            // If the file is smaller than 5MB, upload in one part
            let part = client
                .upload_part()
                .key(&key)
                .bucket(&app_conf.s3_bucket)
                .upload_id(&upload_id)
                .body(ByteStream::from(data))
                .part_number(1)
                .send()
                .await
                .expect("Couldn't upload part");

            let completed_part = CompletedPart::builder()
                .e_tag(part.e_tag().unwrap_or_default())
                .part_number(1)
                .build();

            println!("Completed singular upload: {:?}", completed_part);
        }

        // Wait for all upload parts to complete
    }
}

#[utoipa::path(
    get,
    path = format!("/api/{}", RESOURCE_NAME),
    responses((status = OK, body = super::models::Submission))
)]
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    let (offset, limit) = parse_range(params.range.clone());

    let condition = apply_filters(
        params.filter.clone(),
        &[("name", super::db::Column::Filename)],
    );

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &[
            ("id", super::db::Column::Id),
            ("created_on", super::db::Column::CreatedOn),
            ("filename", super::db::Column::Filename),
            ("size_bytes", super::db::Column::SizeBytes),
        ],
        super::db::Column::Id,
    );

    let objs: Vec<super::db::Model> = super::db::Entity::find()
        .filter(condition.clone())
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(&db)
        .await
        .unwrap();

    // Map the results from the database models
    let response_objs: Vec<super::models::UploadReadOne> =
        objs.into_iter().map(|obj| obj.into()).collect();

    let total_count: u64 = <super::db::Entity>::find()
        .filter(condition.clone())
        .count(&db)
        .await
        .unwrap_or(0);

    let headers = calculate_content_range(offset, limit, total_count, RESOURCE_NAME);

    (headers, Json(response_objs))
}

// #[utoipa::path(
//     post,
//     path = format!("/api/{}", RESOURCE_NAME),
//     responses((status = CREATED, body = super::models::Submission))
// )]
// pub async fn create_one(
//     State(db): State<DatabaseConnection>,
//     Json(payload): Json<super::models::SubmissionCreate>,
// ) -> Result<(StatusCode, Json<super::models::Submission>), (StatusCode, Json<String>)> {
//     let new_obj = super::db::Model {
//         id: uuid::Uuid::new_v4(),
//         name: payload.name,
//         processing_has_started: false,
//         processing_success: false,
//         comment: payload.comment,
//         created_on: chrono::Utc::now().naive_utc(),
//         last_updated: chrono::Utc::now().naive_utc(),
//     }
//     .into_active_model();

//     match super::db::Entity::insert(new_obj).exec(&db).await {
//         Ok(insert_result) => {
//             let response_obj: super::models::Submission =
//                 super::db::Entity::find_by_id(insert_result.last_insert_id)
//                     .one(&db)
//                     .await
//                     .expect("Failed to find object")
//                     .unwrap()
//                     .into();

//             Ok((StatusCode::CREATED, Json(response_obj)))
//         }
//         Err(err) => match err.sql_err() {
//             Some(SqlErr::UniqueConstraintViolation(_)) => {
//                 Err((StatusCode::CONFLICT, Json("Duplicate entry".to_string())))
//             }
//             Some(_) => Err((
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json("Error adding object".to_string()),
//             )),
//             _ => Err((
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json("Server error".to_string()),
//             )),
//         },
//     }
// }

// #[utoipa::path(
//     get,
//     path = format!("/api/{}/{{id}}", RESOURCE_NAME),
//     responses((status = OK, body = super::models::Submission))
// )]
// pub async fn get_one(
//     State(db): State<DatabaseConnection>,
//     Path(id): Path<Uuid>,
// ) -> Result<Json<super::models::Submission>, (StatusCode, Json<String>)> {
//     let obj = super::db::Entity::find_by_id(id)
//         .one(&db)
//         .await
//         .map_err(|_| (StatusCode::NOT_FOUND, Json("Not found".to_string())))?
//         .ok_or((StatusCode::NOT_FOUND, Json("Not found".to_string())))?;

//     Ok(Json(obj.into()))
// }

// #[utoipa::path(
//     put,
//     path = format!("/api/{}/{{id}}", RESOURCE_NAME),
//     responses((status = OK, body = super::models::Submission))
// )]
// pub async fn update_one(
//     State(db): State<DatabaseConnection>,
//     Path(id): Path<Uuid>,
//     Json(payload): Json<super::models::SubmissionUpdate>,
// ) -> impl IntoResponse {
//     let obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
//         .one(&db)
//         .await
//         .unwrap()
//         .expect("Failed to find object")
//         .into();

//     let obj: super::db::ActiveModel = payload.merge_into_activemodel(obj);

//     let obj: super::db::Model = obj.update(&db).await.unwrap();

//     let response_obj: super::models::Submission = obj.into();

//     Json(response_obj)
// }

// // Delete one
// #[utoipa::path(
//     delete,
//     path = format!("/api/{}/{{id}}", RESOURCE_NAME),
//     responses((status = NO_CONTENT))
// )]
// pub async fn delete_one(State(db): State<DatabaseConnection>, Path(id): Path<Uuid>) -> StatusCode {
//     let obj = super::db::Entity::find_by_id(id)
//         .one(&db)
//         .await
//         .unwrap()
//         .expect("Failed to find object");

//     let res: DeleteResult = obj.delete(&db).await.expect("Failed to delete object");

//     if res.rows_affected == 0 {
//         return StatusCode::NOT_FOUND;
//     }

//     StatusCode::NO_CONTENT
// }

// pub async fn check_uploaded_chunks(
//     Extension(db): Extension<DatabaseConnection>,
//     Query(params): Query<HashMap<String, String>>,
//     user: User,
// ) -> impl IntoResponse {
//     let patch = params.get("patch").ok_or((
//         StatusCode::BAD_REQUEST,
//         "Missing 'patch' query parameter".to_string(),
//     ))?;

//     let upload_id = Uuid::parse_str(patch)
//         .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid upload ID: {}", e)))?;

//     // Fetch the upload record
//     let upload = UploadEntity::find()
//         .filter(db::Column::Id.eq(upload_id))
//         .one(&db)
//         .await
//         .map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("Database error: {}", e),
//             )
//         })?
//         .ok_or((StatusCode::NOT_FOUND, "Upload not found".to_string()))?;

//     // Check ownership
//     if upload.owner_id != user.id {
//         return Err((
//             StatusCode::FORBIDDEN,
//             "You do not have permission to access this upload".to_string(),
//         ));
//     }

//     // Calculate the next expected offset
//     let next_expected_offset = if let Some(parts) = &upload.parts {
//         let last_part = parts.as_array().unwrap().last().unwrap();
//         last_part["Offset"].as_i64().unwrap() + last_part["Length"].as_i64().unwrap()
//     } else {
//         0
//     };

//     let mut response = axum::response::Response::new(axum::body::Body::empty());
//     response.headers_mut().insert(
//         "Upload-Offset",
//         next_expected_offset.to_string().parse().unwrap(),
//     );

//     Ok(response)
// }

// pub async fn upload_chunk(
//     Extension(db): Extension<DatabaseConnection>,
//     Extension(s3_client): Extension<Arc<S3Client>>,
//     Query(params): Query<HashMap<String, String>>,
//     mut multipart: Multipart,
//     headers: axum::http::HeaderMap,
//     user: User,
// ) -> impl IntoResponse {
//     let patch = params.get("patch").ok_or((
//         StatusCode::BAD_REQUEST,
//         "Missing 'patch' query parameter".to_string(),
//     ))?;

//     let upload_id = Uuid::parse_str(patch)
//         .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid upload ID: {}", e)))?;

//     // Fetch the upload record
//     let upload = UploadEntity::find()
//         .filter(db::Column::Id.eq(upload_id))
//         .one(&db)
//         .await
//         .map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("Database error: {}", e),
//             )
//         })?
//         .ok_or((StatusCode::NOT_FOUND, "Upload not found".to_string()))?;

//     // Check ownership
//     if upload.owner_id != user.id {
//         return Err((
//             StatusCode::FORBIDDEN,
//             "You do not have permission to access this upload".to_string(),
//         ));
//     }

//     // Extract necessary headers
//     let upload_offset = headers
//         .get("Upload-Offset")
//         .and_then(|h| h.to_str().ok())
//         .and_then(|s| s.parse::<i64>().ok())
//         .ok_or((StatusCode::BAD_REQUEST, "Invalid Upload-Offset".to_string()))?;

//     let upload_length = headers
//         .get("Upload-Length")
//         .and_then(|h| h.to_str().ok())
//         .and_then(|s| s.parse::<i64>().ok())
//         .ok_or((StatusCode::BAD_REQUEST, "Invalid Upload-Length".to_string()))?;

//     let upload_name = headers
//         .get("Upload-Name")
//         .and_then(|h| h.to_str().ok())
//         .unwrap_or("");

//     let content_length = headers
//         .get("Content-Length")
//         .and_then(|h| h.to_str().ok())
//         .and_then(|s| s.parse::<i64>().ok())
//         .ok_or((
//             StatusCode::BAD_REQUEST,
//             "Invalid Content-Length".to_string(),
//         ))?;

//     // Read the data from multipart
//     let mut data = Vec::new();
//     while let Some(field) = multipart.next_field().await.unwrap() {
//         data.extend(field.bytes().await.unwrap());
//     }

//     // Calculate part number
//     let part_number = ((upload_offset / content_length) + 1) as i32;
//     let final_part = upload_offset + content_length == upload_length;

//     // Upload the part to S3
//     let key = format!("inputs/{}", upload.id);
//     let upload_part_output = s3_client
//         .upload_part()
//         .bucket("your-bucket-name")
//         .key(&key)
//         .upload_id(upload.upload_id.as_ref().unwrap())
//         .part_number(part_number)
//         .body(data.into())
//         .send()
//         .await
//         .map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("S3 error: {}", e),
//             )
//         })?;

//     // Update parts in the database
//     let mut parts = upload.parts.unwrap_or_else(|| json!([]));
//     let part_info = json!({
//         "PartNumber": part_number,
//         "ETag": upload_part_output.e_tag.unwrap(),
//         "Size": content_length,
//         "Offset": upload_offset,
//         "Length": content_length
//     });
//     parts.as_array_mut().unwrap().push(part_info);

//     let mut upload_model = upload.into_active_model();
//     upload_model.parts = Set(Some(parts));
//     upload_model.filename = Set(Some(upload_name.to_string()));
//     upload_model.last_part_received_utc = Set(Some(Utc::now().naive_utc()));

//     upload_model.update(&db).await.map_err(|e| {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             format!("Database error: {}", e),
//         )
//     })?;

//     if final_part {
//         // Complete the multipart upload
//         let parts_list = upload_model
//             .parts
//             .as_ref()
//             .unwrap()
//             .as_array()
//             .unwrap()
//             .iter()
//             .map(|p| {
//                 let part_number = p["PartNumber"].as_i64().unwrap() as i32;
//                 let e_tag = p["ETag"].as_str().unwrap().to_string();

//                 CompletedPart::builder()
//                     .part_number(part_number)
//                     .e_tag(e_tag)
//                     .build()
//             })
//             .collect::<Vec<_>>();

//         let completed_multipart_upload = CompletedMultipartUpload::builder()
//             .set_parts(Some(parts_list))
//             .build();

//         s3_client
//             .complete_multipart_upload()
//             .bucket("your-bucket-name")
//             .key(&key)
//             .upload_id(upload_model.upload_id.as_ref().unwrap())
//             .multipart_upload(completed_multipart_upload)
//             .send()
//             .await
//             .map_err(|e| {
//                 (
//                     StatusCode::INTERNAL_SERVER_ERROR,
//                     format!("S3 error: {}", e),
//                 )
//             })?;

//         // Update the database
//         upload_model.all_parts_received = Set(true);
//         upload_model.update(&db).await.map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("Database error: {}", e),
//             )
//         })?;

//         // Start background task for processing
//         // Implement your background task logic here
//     }

//     // Return the updated upload object
//     let updated_upload = UploadEntity::find_by_id(upload_model.id.unwrap())
//         .one(&db)
//         .await
//         .map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("Database error: {}", e),
//             )
//         })?
//         .unwrap();

//     Ok((StatusCode::OK, Json(updated_upload)))
// }

// pub async fn start_upload(
//     Extension(db): Extension<DatabaseConnection>,
//     Extension(s3_client): Extension<Arc<S3Client>>,
//     TypedHeader(content_length): TypedHeader<ContentLength>,
//     TypedHeader(content_type): TypedHeader<ContentType>,
//     headers: axum::http::HeaderMap,
//     user: User,
// ) -> impl IntoResponse {
//     // Extract necessary headers
//     let upload_length = content_length.0 as i64;
//     let transect_id = headers.get("Transect-Id").and_then(|v| v.to_str().ok());

//     // Create a new upload record
//     let new_upload = db::ActiveModel {
//         id: Set(Uuid::new_v4()),
//         size_bytes: Set(upload_length),
//         all_parts_received: Set(false),
//         last_part_received_utc: Set(Some(Utc::now().naive_utc())),
//         processing_message: Set(Some("Upload started".to_string())),
//         owner_id: Set(user.id),
//         created_on: Set(Utc::now().naive_utc()),
//         ..Default::default()
//     };

//     // Insert into the database
//     let upload = new_upload.insert(&db).await.map_err(|e| {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             format!("Database error: {}", e),
//         )
//     })?;

//     // Start S3 multipart upload
//     let key = format!("inputs/{}", upload.id);
//     let create_mpu_output = s3_client
//         .create_multipart_upload()
//         .bucket("your-bucket-name")
//         .key(&key)
//         .send()
//         .await
//         .map_err(|e| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("S3 error: {}", e),
//             )
//         })?;

//     let upload_id = create_mpu_output.upload_id.unwrap();

//     // Update the upload with the S3 upload ID
//     let mut upload_model = upload.into_active_model();
//     upload_model.upload_id = Set(Some(upload_id.clone()));

//     upload_model.update(&db).await.map_err(|e| {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             format!("Database error: {}", e),
//         )
//     })?;

//     // Return the upload ID to the client
//     (StatusCode::OK, upload_model.id.unwrap().to_string())
// }
