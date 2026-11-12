//! User endpoints and the `UsersApi` OpenAPI spec struct.

pub(crate) mod favorites;
pub(crate) mod plays;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, put},
};
use serde::Deserialize;
use uuid::Uuid;

use api_types::{
    common::ErrorResponse,
    pagination::{PagedResponse, defaults as pagination_defaults},
    performances::PerformanceSummary,
    playlists::{PlaylistEntry, PlaylistKind, PlaylistResponse},
    plays::{UserPerformancePlayEntry, UserSoundbitePlayEntry},
    soundbites::SoundbiteResponse,
    users::{GrantCapabilityRequest, UpdateUserRequest, UserSummary},
};
use db::{
    error::DbError,
    queries::{self, playlists::PlaylistFilter},
};

use crate::{
    auth::{middleware::AuthUser, validate_username},
    capabilities, convert,
    error::ApiError,
    pagination,
    routes::{
        common::{SortDir, search_text},
        playlists::params::{PlaylistListParams, PlaylistSort},
        soundbites::params::SoundbiteSort,
    },
    state::AppState,
};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        search_users,
        update_user,
        list_user_playlists,
        get_user_playlists_containing_performance,
        favorites::performances::list_user_favorite_performances,
        favorites::soundbites::list_user_favorite_soundbites,
        favorites::soundbites::list_user_favorite_soundbite_ids,
        favorites::soundbites::add_user_favorite_soundbite,
        favorites::soundbites::remove_user_favorite_soundbite,
        list_user_capabilities,
        grant_capability,
        revoke_capability,
        plays::performances::list_user_performance_plays,
        plays::performances::delete_user_performance_plays,
        plays::performances::delete_user_plays_on_performance,
        plays::soundbites::list_user_soundbite_plays,
        plays::soundbites::delete_user_soundbite_plays,
        plays::soundbites::delete_user_plays_on_soundbite,
    ),
    components(schemas(
        UserPerformancePlayEntry,
        PagedResponse<UserPerformancePlayEntry>,
        UserSoundbitePlayEntry,
        PagedResponse<UserSoundbitePlayEntry>,
        UserSummary,
        PagedResponse<UserSummary>,
        UpdateUserRequest,
        GrantCapabilityRequest,
        PlaylistResponse,
        PlaylistEntry,
        PlaylistKind,
        PlaylistSort,
        SortDir,
        PerformanceSummary,
        SoundbiteResponse,
        SoundbiteSort,
        ErrorResponse,
        PagedResponse<PlaylistResponse>,
        PagedResponse<SoundbiteResponse>,
    ))
)]
pub(crate) struct UsersApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(search_users))
        .route("/{id}", patch(update_user))
        .route("/{id}/playlists", get(list_user_playlists))
        .route(
            "/{id}/playlists/containing/{performance_id}",
            get(get_user_playlists_containing_performance),
        )
        .route(
            "/{id}/favorites/performances",
            get(favorites::performances::list_user_favorite_performances),
        )
        .route(
            "/{id}/favorites/soundbites",
            get(favorites::soundbites::list_user_favorite_soundbites),
        )
        .route(
            "/{id}/favorites/soundbites/ids",
            get(favorites::soundbites::list_user_favorite_soundbite_ids),
        )
        .route(
            "/{id}/favorites/soundbites/{soundbite_id}",
            put(favorites::soundbites::add_user_favorite_soundbite)
                .delete(favorites::soundbites::remove_user_favorite_soundbite),
        )
        .route(
            "/{id}/capabilities",
            get(list_user_capabilities).post(grant_capability),
        )
        .route(
            "/{id}/capabilities/{capability}",
            axum::routing::delete(revoke_capability),
        )
        .route(
            "/{id}/plays/performances",
            get(plays::performances::list_user_performance_plays)
                .delete(plays::performances::delete_user_performance_plays),
        )
        .route(
            "/{id}/plays/performances/{performance_id}",
            axum::routing::delete(plays::performances::delete_user_plays_on_performance),
        )
        .route(
            "/{id}/plays/soundbites",
            get(plays::soundbites::list_user_soundbite_plays)
                .delete(plays::soundbites::delete_user_soundbite_plays),
        )
        .route(
            "/{id}/plays/soundbites/{soundbite_id}",
            axum::routing::delete(plays::soundbites::delete_user_plays_on_soundbite),
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
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "pagination_defaults::page")]
    page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "pagination_defaults::per_page")]
    per_page: u32,
    /// Optional username substring or exact user id filter.
    q: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/users",
    params(UserSearchParams),
    responses(
        (status = 200, description = "Paginated users matching the search query, or all users when `q` is omitted", body = PagedResponse<UserSummary>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn search_users(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<UserSearchParams>,
) -> Result<Json<PagedResponse<UserSummary>>, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::CAPABILITIES_MANAGE)
    {
        return Err(ApiError::Forbidden);
    }
    let (limit, offset) = pagination::limit_offset(params.page, params.per_page);
    let q = search_text(params.q.as_deref());
    let (users, total) = tokio::try_join!(
        queries::users::search(&state.pool, q, limit, offset),
        queries::users::search_count(&state.pool, q),
    )?;

    let items = users
        .into_iter()
        .map(|u| UserSummary {
            id: u.id,
            username: u.username,
        })
        .collect();
    Ok(Json(PagedResponse {
        items,
        total,
        page: params.page,
        per_page: limit,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/users/{id}",
    params(("id" = Uuid, Path, description = "User ID")),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Updated user", body = UserSummary),
        (status = 400, description = "Invalid username", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 409, description = "Username already taken", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserSummary>, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::CAPABILITIES_MANAGE)
    {
        return Err(ApiError::Forbidden);
    }
    validate_username(&req.username)?;

    let user = queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if user.username == req.username {
        return Ok(Json(UserSummary {
            id: user.id,
            username: user.username,
        }));
    }

    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;
    let updated = queries::users::update_username(&mut tx, id, &req.username)
        .await
        .map_err(|e| match e {
            DbError::Conflict => ApiError::Conflict("username already taken".into()),
            other => ApiError::from(other),
        })?;
    queries::playlists::retitle_favorites(&mut *tx, id, &updated.username).await?;
    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok(Json(UserSummary {
        id: updated.id,
        username: updated.username,
    }))
}

#[utoipa::path(
    get,
    path = "/api/users/{id}/playlists",
    params(("id" = Uuid, Path, description = "User ID"), PlaylistListParams),
    responses(
        (status = 200, description = "Playlists for this user. Returns all playlists when viewing your own profile or with sufficient permissions, otherwise public only.", body = PagedResponse<PlaylistResponse>),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
pub(crate) async fn list_user_playlists(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<PlaylistListParams>,
    auth: Option<AuthUser>,
) -> Result<Json<PagedResponse<PlaylistResponse>>, ApiError> {
    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let include_private = can_view_private(&auth, id);
    let (limit, offset) = pagination::limit_offset(params.page, params.per_page);
    let q = search_text(params.q.as_deref());
    let kind = params.kind.as_ref().map(PlaylistKind::as_str);
    let (sort, direction) = params.order();
    let filter = PlaylistFilter {
        q,
        include_private,
        created_by: Some(id),
        kind,
    };
    let (rows, total) = tokio::try_join!(
        queries::playlists::search(&state.pool, filter, sort, direction, limit, offset),
        queries::playlists::search_count(&state.pool, filter),
    )?;
    let items = rows
        .into_iter()
        .map(convert::playlist_response)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(PagedResponse {
        items,
        total,
        page: params.page,
        per_page: limit,
    }))
}

#[utoipa::path(
    get,
    path = "/api/users/{id}/playlists/containing/{performance_id}",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        ("performance_id" = Uuid, Path, description = "Performance ID"),
    ),
    responses(
        (status = 200, description = "IDs of this user's playlists that contain the given performance.", body = Vec<Uuid>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn get_user_playlists_containing_performance(
    State(state): State<AppState>,
    Path((id, performance_id)): Path<(Uuid, Uuid)>,
    auth: AuthUser,
) -> Result<Json<Vec<Uuid>>, ApiError> {
    if auth.user_id != id
        && !auth
            .capabilities
            .contains(capabilities::PLAYLISTS_VIEW_PRIVATE)
    {
        return Err(ApiError::Forbidden);
    }
    let ids = queries::playlists::get_user_playlist_ids_containing_performance(
        &state.pool,
        id,
        performance_id,
    )
    .await?;
    Ok(Json(ids))
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
    tag = "users",
    security(("session" = []))
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
    tag = "users",
    security(("session" = []))
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
    tag = "users",
    security(("session" = []))
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
