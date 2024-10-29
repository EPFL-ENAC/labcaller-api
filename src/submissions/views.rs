use crate::common::auth::Role;
use crate::common::filter::{apply_filters, parse_range};
use crate::common::models::FilterOptions;
use crate::common::pagination::calculate_content_range;
use crate::common::sort::generic_sort;
use crate::external::k8s::crd::{
    Environment, EnvironmentItems, TrainingWorkload, TrainingWorkloadSpec, ValueField,
};
use anyhow::Result;
use axum::{
    debug_handler,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use axum_keycloak_auth::{
    instance::KeycloakAuthInstance, layer::KeycloakAuthLayer, PassthroughMode,
};
use kube::{api::PostParams, Api};
use rand::Rng;
use sea_orm::{
    query::*, ActiveModelTrait, DatabaseConnection, DeleteResult, EntityTrait, IntoActiveModel,
    ModelTrait, SqlErr,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn router(db: DatabaseConnection, keycloak_auth_instance: Arc<KeycloakAuthInstance>) -> Router {
    Router::new()
        .route("/", routing::get(get_all).post(create_one))
        .route(
            "/:id",
            routing::get(get_one)
                .put(update_one)
                .delete(delete_one)
                .post(execute_workflow),
        )
        .with_state(db)
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

#[utoipa::path(
    post,
    path = format!("/api/{}", RESOURCE_NAME),
    responses((status = CREATED, body = super::models::Submission))
)]
pub async fn create_one(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<super::models::SubmissionCreate>,
) -> Result<(StatusCode, Json<super::models::Submission>), (StatusCode, Json<String>)> {
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
            let response_obj: super::models::Submission =
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
    responses((status = OK, body = super::models::Submission))
)]
pub async fn get_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<super::models::Submission>, (StatusCode, Json<String>)> {
    let obj = match super::db::Entity::find_by_id(id)
        // .find_also_related(crate::uploads::associations::db::Entity)
        .one(&db)
        .await
    {
        Ok(obj) => obj.unwrap(),
        _ => return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
    };

    let uploads = match obj.find_related(crate::uploads::db::Entity).all(&db).await {
        // Return all or none. If any fail, return an error
        Ok(uploads) => Some(uploads),
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Server error".to_string()),
            ))
        }
    };

    let submission: super::models::Submission = (obj, uploads).into();

    Ok(Json(submission))
}

#[utoipa::path(
    put,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = OK, body = super::models::Submission))
)]
pub async fn update_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<super::models::SubmissionUpdate>,
) -> impl IntoResponse {
    let obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
        .one(&db)
        .await
        .unwrap()
        .expect("Failed to find object")
        .into();

    let obj: super::db::ActiveModel = payload.merge_into_activemodel(obj);

    let obj: super::db::Model = obj.update(&db).await.unwrap();

    let response_obj: super::models::Submission = obj.into();

    Json(response_obj)
}

// Delete one
#[utoipa::path(
    delete,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = NO_CONTENT))
)]
pub async fn delete_one(State(db): State<DatabaseConnection>, Path(id): Path<Uuid>) -> StatusCode {
    let obj = super::db::Entity::find_by_id(id)
        .one(&db)
        .await
        .unwrap()
        .expect("Failed to find object");

    let res: DeleteResult = obj.delete(&db).await.expect("Failed to delete object");

    if res.rows_affected == 0 {
        return StatusCode::NOT_FOUND;
    }

    StatusCode::NO_CONTENT
}

#[debug_handler]
pub async fn execute_workflow(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    // Generate a unique job name
    let random_number: u32 = rand::thread_rng().gen_range(10000..99999);
    let job_name = format!("labcaller-{}-{}", id, random_number);

    // Fetch submission and related uploads
    let obj = match super::db::Entity::find_by_id(id).one(&db).await {
        Ok(Some(submission)) => submission,
        _ => return StatusCode::NOT_FOUND,
    };

    let input_object_ids: Vec<Uuid> = obj
        .find_related(crate::uploads::db::Entity)
        .all(&db)
        .await
        .unwrap()
        .into_iter()
        .map(|assoc| assoc.id)
        .collect();

    // Set up Kubernetes client and configuration
    let config = crate::config::Config::from_env();
    let client = match crate::external::k8s::services::refresh_token_and_get_client().await {
        Ok(client) => client,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let base_image = format!(
        "{}:{}",
        config.submission_base_image, config.submission_base_image_tag,
    );
    // Create a new TrainingWorkload custom resource
    let training_workload = TrainingWorkload::new(
        &job_name,
        TrainingWorkloadSpec {
            allow_privilege_escalation: Some(ValueField { value: true }),
            environment: Environment {
                items: EnvironmentItems {
                    input_object_ids: ValueField {
                        value: serde_json::to_string(&input_object_ids).unwrap(),
                    },
                    s3_access_key: ValueField {
                        value: config.s3_access_key.to_string(),
                    },
                    s3_bucket_id: ValueField {
                        value: config.s3_bucket.to_string(),
                    },
                    s3_prefix: ValueField {
                        value: config.s3_prefix.to_string(),
                    },
                    s3_secret_key: ValueField {
                        value: config.s3_secret_key.to_string(),
                    },
                    s3_url: ValueField {
                        value: config.s3_url.to_string(),
                    },
                    submission_id: ValueField {
                        value: id.to_string(),
                    },
                    base_image: ValueField {
                        value: base_image.clone(),
                    },
                },
            },
            gpu: ValueField {
                value: "1".to_string(),
            },
            image: ValueField { value: base_image },
            image_pull_policy: ValueField {
                value: "Always".to_string(),
            },
            name: ValueField {
                value: job_name.clone(),
            },
            run_as_gid: None,
            run_as_uid: None,
            run_as_user: None,
            service_type: None,
            usage: Some("Submit".to_string()),
        },
    );

    println!("Submitting TrainingWorkload: {:?}", training_workload);
    // Submit the custom resource to Kubernetes
    let api: Api<TrainingWorkload> = Api::namespaced(client, &config.kube_namespace);

    match api.create(&PostParams::default(), &training_workload).await {
        Ok(_) => {
            println!("Submitted TrainingWorkload: {}", job_name);
            StatusCode::CREATED
        }
        Err(e) => {
            eprintln!("Failed to submit TrainingWorkload: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
