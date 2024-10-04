#[utoipa::path(
    get,
    path = "/api/healthz",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
pub async fn healthz() -> &'static str {
    // Get health of the API.
    "ok"
}
