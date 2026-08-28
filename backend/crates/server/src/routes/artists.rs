//! Artist CRUD handlers and the `ArtistsApi` OpenAPI spec struct.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use api_types::{
    artists::{ArtistResponse, CreateArtistRequest, UpdateArtistRequest},
    common::ErrorResponse,
    pagination::{PagedResponse, PaginationParams},
};
use db::{
    error::DbError,
    models::{NewArtist, UpdateArtist},
    queries,
};

use crate::{
    auth::middleware::AuthUser, capabilities, convert, error::ApiError, pagination, state::AppState,
};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_artists, get_artist, create_artist, update_artist, delete_artist),
    components(schemas(
        ArtistResponse,
        CreateArtistRequest,
        UpdateArtistRequest,
        ErrorResponse,
        PagedResponse<ArtistResponse>,
    ))
)]
pub(crate) struct ArtistsApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_artists).post(create_artist))
        .route(
            "/{id}",
            get(get_artist).put(update_artist).delete(delete_artist),
        )
}

#[utoipa::path(
    get,
    path = "/api/artists",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of artists ordered by name", body = PagedResponse<ArtistResponse>),
    ),
    tag = "artists"
)]
pub(crate) async fn list_artists(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PagedResponse<ArtistResponse>>, ApiError> {
    let (limit, offset) = pagination::limit_offset(&params);
    let (artists, total) = tokio::try_join!(
        queries::artists::list(&state.pool, limit, offset),
        queries::artists::count(&state.pool),
    )?;
    Ok(Json(PagedResponse {
        items: artists.into_iter().map(convert::artist_response).collect(),
        total,
        page: params.page,
        per_page: limit,
    }))
}

#[utoipa::path(
    get,
    path = "/api/artists/{id}",
    params(("id" = Uuid, Path, description = "Artist ID")),
    responses(
        (status = 200, description = "Artist detail", body = ArtistResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "artists"
)]
pub(crate) async fn get_artist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ArtistResponse>, ApiError> {
    let artist = queries::artists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(convert::artist_response(artist)))
}

#[utoipa::path(
    post,
    path = "/api/artists",
    request_body = CreateArtistRequest,
    responses(
        (status = 201, description = "Created artist", body = ArtistResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "artists"
)]
pub(crate) async fn create_artist(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateArtistRequest>,
) -> Result<(StatusCode, Json<ArtistResponse>), ApiError> {
    if !auth.capabilities.contains(capabilities::ARTISTS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let artist = queries::artists::create(
        &mut conn,
        &NewArtist {
            name: req.name,
            description: req.description,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(convert::artist_response(artist))))
}

#[utoipa::path(
    put,
    path = "/api/artists/{id}",
    params(("id" = Uuid, Path, description = "Artist ID")),
    request_body = UpdateArtistRequest,
    responses(
        (status = 200, description = "Updated artist", body = ArtistResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "artists"
)]
pub(crate) async fn update_artist(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateArtistRequest>,
) -> Result<Json<ArtistResponse>, ApiError> {
    if !auth.capabilities.contains(capabilities::ARTISTS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let artist = queries::artists::update(
        &mut conn,
        id,
        &UpdateArtist {
            name: req.name,
            description: req.description,
        },
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(convert::artist_response(artist)))
}

#[utoipa::path(
    delete,
    path = "/api/artists/{id}",
    params(("id" = Uuid, Path, description = "Artist ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "artists"
)]
pub(crate) async fn delete_artist(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !auth.capabilities.contains(capabilities::ARTISTS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let found = queries::artists::delete(&state.pool, id).await?;
    if found {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}
