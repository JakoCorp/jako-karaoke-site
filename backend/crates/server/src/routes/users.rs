//! User endpoints and the `UsersApi` OpenAPI spec struct.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use api_types::{
    common::ErrorResponse,
    performances::PerformanceSummary,
    playlists::{PlaylistKind, PlaylistResponse},
    users::{GrantCapabilityRequest, UserSummary},
};
use db::queries;

use crate::{auth::middleware::AuthUser, capabilities, convert, error::ApiError, state::AppState};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        search_users,
        list_user_playlists,
        get_user_favorites,
        list_user_capabilities,
        grant_capability,
        revoke_capability,
    ),
    components(schemas(
        UserSummary,
        GrantCapabilityRequest,
        PlaylistResponse,
        PlaylistKind,
        PerformanceSummary,
        ErrorResponse,
    ))
)]
pub(crate) struct UsersApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(search_users))
        .route("/{id}/playlists", get(list_user_playlists))
        .route("/{id}/favorites", get(get_user_favorites))
        .route(
            "/{id}/capabilities",
            get(list_user_capabilities).post(grant_capability),
        )
        .route(
            "/{id}/capabilities/{capability}",
            axum::routing::delete(revoke_capability),
        )
}

fn can_view_private(auth: &Option<AuthUser>, user_id: Uuid) -> bool {
    auth.as_ref().is_some_and(|u| {
        u.user_id == user_id
            || u.capabilities
                .contains(capabilities::PLAYLISTS_VIEW_PRIVATE)
    })
}

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct UserSearchParams {
    /// Optional username substring filter.
    q: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/users",
    params(UserSearchParams),
    responses(
        (status = 200, description = "Users matching the search query, or all users when `q` is omitted", body = Vec<UserSummary>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "users"
)]
pub(crate) async fn search_users(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<UserSearchParams>,
) -> Result<Json<Vec<UserSummary>>, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::CAPABILITIES_MANAGE)
    {
        return Err(ApiError::Forbidden);
    }
    let results = queries::users::search(&state.pool, params.q.as_deref()).await?;
    Ok(Json(
        results
            .into_iter()
            .map(|u| UserSummary {
                id: u.id,
                username: u.username,
            })
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/users/{id}/playlists",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "Playlists for this user. Returns all playlists when viewing your own profile or with sufficient permissions, otherwise public only.", body = Vec<PlaylistResponse>),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
pub(crate) async fn list_user_playlists(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: Option<AuthUser>,
) -> Result<Json<Vec<PlaylistResponse>>, ApiError> {
    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let playlists = if can_view_private(&auth, id) {
        queries::playlists::list_by_user(&state.pool, id).await?
    } else {
        queries::playlists::list_public_by_user(&state.pool, id).await?
    };

    let items = playlists
        .into_iter()
        .map(convert::playlist_response)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(items))
}

#[utoipa::path(
    get,
    path = "/api/users/{id}/favorites",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "Ordered performances in this user's favorites playlist.", body = Vec<PerformanceSummary>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
pub(crate) async fn get_user_favorites(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: Option<AuthUser>,
) -> Result<Json<Vec<PerformanceSummary>>, ApiError> {
    if !can_view_private(&auth, id) {
        return Err(ApiError::Forbidden);
    }

    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let playlist = queries::playlists::get_favorites_by_user(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let performances =
        queries::playlists::get_performances_in_playlist(&state.pool, playlist.id).await?;
    let items =
        crate::routes::performances::build_performance_summaries(&state.pool, performances).await?;

    Ok(Json(items))
}

#[utoipa::path(
    get,
    path = "/api/users/{id}/capabilities",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "Capabilities assigned to this user", body = Vec<String>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
pub(crate) async fn list_user_capabilities(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<String>>, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::CAPABILITIES_MANAGE)
    {
        return Err(ApiError::Forbidden);
    }
    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let caps = queries::capabilities::list_for_user(&state.pool, id).await?;
    Ok(Json(caps))
}

#[utoipa::path(
    post,
    path = "/api/users/{id}/capabilities",
    params(("id" = Uuid, Path, description = "User ID")),
    request_body = GrantCapabilityRequest,
    responses(
        (status = 204, description = "Capability granted"),
        (status = 400, description = "Unknown capability", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
pub(crate) async fn grant_capability(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<GrantCapabilityRequest>,
) -> Result<StatusCode, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::CAPABILITIES_MANAGE)
    {
        return Err(ApiError::Forbidden);
    }
    if !capabilities::ALL_CAPABILITIES.contains(&req.capability.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "unknown capability: {}",
            req.capability
        )));
    }
    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(db::error::DbError::Sqlx)?;
    queries::capabilities::add_to_user(&mut conn, id, &req.capability).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/users/{id}/capabilities/{capability}",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        ("capability" = String, Path, description = "Capability string to revoke"),
    ),
    responses(
        (status = 204, description = "Capability revoked"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
pub(crate) async fn revoke_capability(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, capability)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::CAPABILITIES_MANAGE)
    {
        return Err(ApiError::Forbidden);
    }
    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    queries::capabilities::remove_from_user(&state.pool, id, &capability).await?;
    Ok(StatusCode::NO_CONTENT)
}
