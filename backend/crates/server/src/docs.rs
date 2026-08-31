//! OpenAPI spec assembly and Swagger UI endpoint.

use axum::Router;
use utoipa::OpenApi;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    auth::AuthApi,
    routes::{
        artists::ArtistsApi, capabilities::CapabilitiesApi, performances::PerformancesApi,
        playlists::PlaylistsApi, songs::SongsApi, tags::TagsApi, users::UsersApi,
    },
    state::AppState,
};

/// Assembles the merged OpenAPI spec from all API groups.
pub fn openapi_spec() -> utoipa::openapi::OpenApi {
    let mut spec = ArtistsApi::openapi();
    spec.merge(CapabilitiesApi::openapi());
    spec.merge(PerformancesApi::openapi());
    spec.merge(PlaylistsApi::openapi());
    spec.merge(SongsApi::openapi());
    spec.merge(TagsApi::openapi());
    spec.merge(UsersApi::openapi());
    spec.merge(AuthApi::openapi());
    spec.components
        .get_or_insert_with(Default::default)
        .add_security_scheme(
            "session",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("session"))),
        );
    spec
}

/// Builds the `/docs` router, serving the merged OpenAPI spec and Swagger UI.
pub fn router() -> Router<AppState> {
    Router::new().merge(SwaggerUi::new("/docs").url("/docs/openapi.json", openapi_spec()))
}
