use crate::common::filter::{apply_filters, parse_range};
use crate::common::models::FilterOptions;
use crate::common::pagination::calculate_content_range;
use crate::common::sort::generic_sort;
use axum::http::{response, StatusCode};
use axum::response::IntoResponse;
use axum::{
    debug_handler,
    extract::{Path, Query, State},
    routing, Json, Router,
};
use sea_orm::{query::*, DatabaseConnection, DbErr, IntoActiveModel, SqlErr, SqlxError};
use sea_orm::{EntityTrait, RuntimeErr};
use std::borrow::Cow;
use uuid::Uuid;

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_all).post(create_one))
        .route("/:id", routing::get(get_one).put(update_one))
        .with_state(db)
}

const RESOURCE_NAME: &str = "submissions";

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

    let condition = apply_filters(params.filter.clone(), &[("name", super::db::Column::Name)]);

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &[
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
            (
                "processing_has_started",
                super::db::Column::ProcessingHasStarted,
            ),
            ("processing_success", super::db::Column::ProcessingSuccess),
            ("comment", super::db::Column::Comment),
            ("created_on", super::db::Column::CreatedOn),
            ("last_updated", super::db::Column::LastUpdated),
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
    let response_objs: Vec<super::models::Submission> =
        objs.into_iter().map(|obj| obj.into()).collect();

    let total_count: u64 = <super::db::Entity>::find()
        .filter(condition.clone())
        .count(&db)
        .await
        .unwrap_or(0);

    let headers = calculate_content_range(offset, limit, total_count, RESOURCE_NAME);

    (headers, Json(response_objs))
}

pub async fn create_one(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<super::models::SubmissionCreate>,
) -> Result<(StatusCode, Json<super::models::SubmissionReadOne>), (StatusCode, Json<String>)> {
    let new_obj = super::db::Model {
        id: uuid::Uuid::new_v4(),
        name: payload.name,
        processing_has_started: false,
        processing_success: false,
        comment: payload.comment,
        created_on: chrono::Utc::now().naive_utc(),
        last_updated: chrono::Utc::now().naive_utc(),
    }
    .into_active_model();

    match super::db::Entity::insert(new_obj).exec(&db).await {
        Ok(insert_result) => {
            let response_obj: super::models::SubmissionReadOne =
                super::db::Entity::find_by_id(insert_result.last_insert_id)
                    .one(&db)
                    .await
                    .expect("Failed to find object")
                    .unwrap()
                    .into();

            Ok((StatusCode::CREATED, Json(response_obj)))
        }
        Err(err) => match err.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                Err((StatusCode::CONFLICT, Json("Duplicate entry".to_string())))
            }
            Some(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error adding object".to_string()),
            )),
            _ => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Server error".to_string()),
            )),
        },
    }
}

#[utoipa::path(
    get,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = OK, body = super::models::SubmissionReadOne))
)]
#[debug_handler]
pub async fn get_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<super::models::SubmissionReadOne>, (StatusCode, Json<String>)> {
    let obj = super::db::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, Json("Not found".to_string())))?
        .ok_or((StatusCode::NOT_FOUND, Json("Not found".to_string())))?;

    Ok(Json(obj.into()))
}

#[utoipa::path(
    put,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = OK, body = super::models::SubmissionReadOne))
)]
pub async fn update_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<super::models::SubmissionUpdate>,
) -> impl IntoResponse {
    // ) -> Result<Json<super::models::SubmissionReadOne>, (StatusCode, Json<String>)> {
    println!("Update one {:?} {:?}", payload.name, payload.comment);
    let update_object: super::db::ActiveModel = payload.into();

    let obj = super::db::Entity::find_by_id(id)
        .one(&db)
        .await
        .unwrap()
        .expect("Failed to find object");

    // Update obj with the new values from update_object

    // .unwrap()
    // .unwrap();

    // Update the object
    // obj.update(&db).await;

    println!("Update one {:?}", obj);
    println!("Update object {:?}", update_object);

    ()
}
