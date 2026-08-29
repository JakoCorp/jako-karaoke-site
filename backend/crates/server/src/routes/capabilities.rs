//! Capability reference endpoints and the `CapabilitiesApi` OpenAPI spec struct.

use axum::{Json, Router, routing::get};

use api_types::common::ErrorResponse;

use crate::{auth::middleware::AuthUser, capabilities, error::ApiError, state::AppState};

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_capabilities), components(schemas(ErrorResponse)))]
pub(crate) struct CapabilitiesApi;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_capabilities))
}

#[utoipa::path(
    get,
    path = "/api/capabilities",
    responses(
        (status = 200, description = "All capability strings known to this server", body = Vec<String>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "capabilities",
    security(("session" = []))
)]
pub(crate) async fn list_capabilities(auth: AuthUser) -> Result<Json<Vec<String>>, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::CAPABILITIES_MANAGE)
    {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(
        capabilities::ALL_CAPABILITIES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    ))
}
