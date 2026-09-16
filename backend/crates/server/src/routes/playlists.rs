//! Playlist CRUD handlers and the `PlaylistsApi` OpenAPI spec struct.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use api_types::{
    common::ErrorResponse,
    performances::PerformanceSummary,
    playlists::{
        AddPerformancesRequest, CreatePlaylistRequest, PlaylistEntry, PlaylistKind,
        PlaylistResponse, RemovePerformancesRequest, UpdatePlaylistRequest,
    },
};
use db::{
    error::DbError,
    models::playlist::{NewPlaylist, UpdatePlaylist},
    queries,
};

use crate::{auth::middleware::AuthUser, capabilities, convert, error::ApiError, state::AppState};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_playlists,
        get_playlist,
        create_playlist,
        update_playlist,
        delete_playlist,
        list_playlist_performances,
        add_playlist_performances,
        remove_playlist_performances,
    ),
    components(schemas(
        PlaylistResponse,
        PlaylistEntry,
        PlaylistKind,
        CreatePlaylistRequest,
        UpdatePlaylistRequest,
        AddPerformancesRequest,
        RemovePerformancesRequest,
        PerformanceSummary,
        ErrorResponse,
    ))
)]
pub(crate) struct PlaylistsApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_playlists).post(create_playlist))
        .route(
            "/{id}",
            get(get_playlist)
                .put(update_playlist)
                .delete(delete_playlist),
        )
        .route(
            "/{id}/performances",
            get(list_playlist_performances)
                .post(add_playlist_performances)
                .delete(remove_playlist_performances),
        )
}

#[utoipa::path(
    get,
    path = "/api/playlists",
    responses(
        (status = 200, description = "Returns public playlists, or all playlists for users with sufficient permissions.", body = Vec<PlaylistResponse>),
    ),
    tag = "playlists"
)]
pub(crate) async fn list_playlists(
    State(state): State<AppState>,
    auth: Option<AuthUser>,
) -> Result<Json<Vec<PlaylistResponse>>, ApiError> {
    let can_view_private = auth.is_some_and(|u| {
        u.capabilities
            .contains(capabilities::PLAYLISTS_VIEW_PRIVATE)
    });
    let playlists = if can_view_private {
        queries::playlists::list_all(&state.pool).await?
    } else {
        queries::playlists::list_public(&state.pool).await?
    };
    let items = playlists
        .into_iter()
        .map(convert::playlist_response)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(items))
}

#[utoipa::path(
    get,
    path = "/api/playlists/{id}",
    params(("id" = Uuid, Path, description = "Playlist ID")),
    responses(
        (status = 200, description = "Playlist detail", body = PlaylistResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "playlists"
)]
pub(crate) async fn get_playlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: Option<AuthUser>,
) -> Result<Json<PlaylistResponse>, ApiError> {
    let playlist = queries::playlists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if !playlist.is_public {
        let can_view = auth.as_ref().is_some_and(|u| {
            playlist.created_by == Some(u.user_id)
                || u.capabilities
                    .contains(capabilities::PLAYLISTS_VIEW_PRIVATE)
        });
        if !can_view {
            return Err(ApiError::Forbidden);
        }
    }

    Ok(Json(convert::playlist_response(playlist)?))
}

#[utoipa::path(
    post,
    path = "/api/playlists",
    request_body = CreatePlaylistRequest,
    responses(
        (status = 201, description = "Created playlist", body = PlaylistResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "playlists",
    security(("session" = []))
)]
pub(crate) async fn create_playlist(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreatePlaylistRequest>,
) -> Result<(StatusCode, Json<PlaylistResponse>), ApiError> {
    if let Some(cap) = capabilities::required_playlist_create_capability(&req.kind)
        && !auth.capabilities.contains(cap)
    {
        return Err(ApiError::Forbidden);
    }

    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let playlist = queries::playlists::create(
        &mut conn,
        &NewPlaylist {
            title: req.title,
            description: req.description,
            kind: req.kind.as_str().to_owned(),
            is_public: req.is_public,
            created_by: Some(auth.user_id),
        },
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(convert::playlist_response(playlist)?),
    ))
}

#[utoipa::path(
    put,
    path = "/api/playlists/{id}",
    params(("id" = Uuid, Path, description = "Playlist ID")),
    request_body = UpdatePlaylistRequest,
    responses(
        (status = 200, description = "Updated playlist", body = PlaylistResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "playlists",
    security(("session" = []))
)]
pub(crate) async fn update_playlist(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePlaylistRequest>,
) -> Result<Json<PlaylistResponse>, ApiError> {
    let existing = queries::playlists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let can_manage = existing.created_by == Some(auth.user_id)
        || auth
            .capabilities
            .contains(capabilities::PLAYLISTS_MANAGE_ANY);
    if !can_manage {
        return Err(ApiError::Forbidden);
    }

    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let playlist = queries::playlists::update(
        &mut conn,
        id,
        &UpdatePlaylist {
            title: req.title,
            description: req.description,
            kind: req.kind.as_str().to_owned(),
            is_public: req.is_public,
        },
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(convert::playlist_response(playlist)?))
}

#[utoipa::path(
    delete,
    path = "/api/playlists/{id}",
    params(("id" = Uuid, Path, description = "Playlist ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "playlists",
    security(("session" = []))
)]
pub(crate) async fn delete_playlist(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let playlist = queries::playlists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let can_manage = playlist.created_by == Some(auth.user_id)
        || auth
            .capabilities
            .contains(capabilities::PLAYLISTS_MANAGE_ANY);
    if !can_manage {
        return Err(ApiError::Forbidden);
    }

    queries::playlists::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/playlists/{id}/performances",
    params(("id" = Uuid, Path, description = "Playlist ID")),
    responses(
        (status = 200, description = "Ordered performances in this playlist", body = Vec<PlaylistEntry>),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "playlists"
)]
pub(crate) async fn list_playlist_performances(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<PlaylistEntry>>, ApiError> {
    queries::playlists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let rows = queries::playlists::get_performances_in_playlist(&state.pool, id).await?;
    let added_ats: Vec<_> = rows.iter().map(|r| r.added_at).collect();
    let performances: Vec<_> = rows.into_iter().map(|r| r.performance).collect();
    let summaries =
        crate::routes::performances::build_performance_summaries(&state.pool, performances).await?;
    let items = summaries
        .into_iter()
        .zip(added_ats)
        .map(|(summary, added_at)| convert::playlist_entry(summary, added_at))
        .collect();

    Ok(Json(items))
}

#[utoipa::path(
    post,
    path = "/api/playlists/{id}/performances",
    params(("id" = Uuid, Path, description = "Playlist ID")),
    request_body = AddPerformancesRequest,
    responses(
        (status = 204, description = "Performances added"),
        (status = 400, description = "Playlist would exceed the 1000-entry limit", body = ErrorResponse),
        (status = 404, description = "Playlist not found", body = ErrorResponse),
    ),
    tag = "playlists"
)]
pub(crate) async fn add_playlist_performances(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddPerformancesRequest>,
) -> Result<StatusCode, ApiError> {
    let playlist = queries::playlists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if playlist.performance_count as u64 + req.performance_ids.len() as u64 > 1000 {
        return Err(ApiError::BadRequest(
            "playlist would exceed the 1000-entry limit".to_owned(),
        ));
    }
    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    queries::playlists::add_performances(&mut conn, id, &req.performance_ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/playlists/{id}/performances",
    params(("id" = Uuid, Path, description = "Playlist ID")),
    request_body = RemovePerformancesRequest,
    responses(
        (status = 204, description = "Performances removed"),
        (status = 404, description = "Playlist not found", body = ErrorResponse),
    ),
    tag = "playlists"
)]
pub(crate) async fn remove_playlist_performances(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<RemovePerformancesRequest>,
) -> Result<StatusCode, ApiError> {
    queries::playlists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    queries::playlists::remove_performances(&state.pool, id, &req.performance_ids).await?;
    Ok(StatusCode::NO_CONTENT)
}
