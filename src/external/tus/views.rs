use super::hooks::{
    // handle_post_create,
    // handle_post_finish,
    handle_post_create,
    handle_post_finish,
    handle_post_receive,
    handle_pre_create,
    handle_pre_finish, // handle_pre_finish,
};
use crate::external::tus::models::{EventPayload, EventType};
// use crate::objects::models::InputObject;
use super::models::PreCreateResponse;
use crate::common::auth::Role;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_keycloak_auth::{
    instance::KeycloakAuthInstance, layer::KeycloakAuthLayer, PassthroughMode,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub fn router(db: DatabaseConnection, keycloak_auth_instance: Arc<KeycloakAuthInstance>) -> Router {
    Router::new()
        .route("/hooks", post(handle_tus_hooks))
        .with_state(db)
    // .layer(DefaultBodyLimit::max(1073741824))
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

// Example of async function to handle tus hook events
#[axum::debug_handler]
pub async fn handle_tus_hooks(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<EventPayload>,
) -> (StatusCode, Json<PreCreateResponse>) {
    match payload.event_type {
        EventType::PreCreate => match handle_pre_create(db, payload).await {
            Ok(response) => (StatusCode::CREATED, Json(response)),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PreCreateResponse {
                    change_file_info: None,
                    status: "error".to_string(),
                }),
            ),
        },
        EventType::PostReceive => match handle_post_receive(db, payload).await {
            Ok(response) => (StatusCode::CREATED, Json(response)),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PreCreateResponse {
                    change_file_info: None,
                    status: "error".to_string(),
                }),
            ),
        },
        EventType::PostCreate => match handle_post_create(db, payload).await {
            Ok(response) => (StatusCode::CREATED, Json(response)),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PreCreateResponse {
                    change_file_info: None,
                    status: "error".to_string(),
                }),
            ),
        },
        EventType::PreFinish => match handle_pre_finish(db, payload).await {
            Ok(response) => (StatusCode::CREATED, Json(response)),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PreCreateResponse {
                    change_file_info: None,
                    status: "error".to_string(),
                }),
            ),
        },

        EventType::PostFinish => match handle_post_finish(db, payload).await {
            Ok(response) => (StatusCode::CREATED, Json(response)),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PreCreateResponse {
                    change_file_info: None,
                    status: "error".to_string(),
                }),
            ),
        },
        _ => {
            println!("Unknown event type {:?}", payload.event_type);
            (
                StatusCode::NOT_FOUND,
                Json(PreCreateResponse {
                    change_file_info: None,
                    status: "Not found".to_string(),
                }),
            )
        }
    }
}
