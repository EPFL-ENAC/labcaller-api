use crate::common::auth::Role;
use crate::common::filter::{apply_filters, parse_range};
use crate::common::models::FilterOptions;
use crate::common::pagination::calculate_content_range;
use crate::common::sort::generic_sort;
use aws_sdk_s3::Client as S3Client;
use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use axum_keycloak_auth::{
    instance::KeycloakAuthInstance, layer::KeycloakAuthLayer, PassthroughMode,
};
use sea_orm::{query::*, DatabaseConnection, EntityTrait};
use std::sync::Arc;
use uuid::Uuid;

const RESOURCE_NAME: &str = "uploads";

pub fn router(
    db: DatabaseConnection,
    keycloak_auth_instance: Arc<KeycloakAuthInstance>,
    s3: Arc<S3Client>,
) -> Router {
    Router::new()
        .route("/", routing::get(get_all))
        .route("/:id", routing::get(get_one).delete(delete_one))
        .with_state((db, s3))
        .layer(DefaultBodyLimit::max(1073741824))
        .layer(
            KeycloakAuthLayer::<Role>::builder()
                .instance(keycloak_auth_instance)
                .passthrough_mode(PassthroughMode::Block)
                .persist_raw_claims(false)
                .expected_audiences(vec![String::from("account")])
                .required_roles(vec![Role::Administrator])
                .build(),
        )
}

#[utoipa::path(
    get,
    path = format!("/api/{}", RESOURCE_NAME),
    responses((status = OK, body = super::models::Submission))
)]
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State((db, _s3)): State<(DatabaseConnection, Arc<S3Client>)>,
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
            ("all_parts_received", super::db::Column::AllPartsReceived),
            ("last_part_received", super::db::Column::LastPartReceived),
            ("processing_message", super::db::Column::ProcessingMessage),
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
    let response_objs: Vec<super::models::UploadRead> =
        objs.into_iter().map(|obj| obj.into()).collect();

    let total_count: u64 = <super::db::Entity>::find()
        .filter(condition.clone())
        .count(&db)
        .await
        .unwrap_or(0);

    let headers = calculate_content_range(offset, limit, total_count, RESOURCE_NAME);

    (headers, Json(response_objs))
}

#[utoipa::path(
    get,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = OK, body = super::models::Submission))
)]
pub async fn get_one(
    State((db, _s3)): State<(DatabaseConnection, Arc<S3Client>)>,
    Path(id): Path<Uuid>,
) -> Result<Json<super::models::UploadRead>, (StatusCode, Json<String>)> {
    let obj = super::db::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, Json("Not found".to_string())))?
        .ok_or((StatusCode::NOT_FOUND, Json("Not found".to_string())))?;

    Ok(Json(obj.into()))
}

#[utoipa::path(
    delete,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = NO_CONTENT))
)]
pub async fn delete_one(
    State((db, s3)): State<(DatabaseConnection, Arc<S3Client>)>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    match super::services::delete_object(&db, &s3, id).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(err) => {
            // Log the error if needed
            if err.to_string() == "Object not found" {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
