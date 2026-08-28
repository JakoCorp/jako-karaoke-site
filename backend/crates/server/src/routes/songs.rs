//! Song CRUD handlers and the `SongsApi` OpenAPI spec struct.

pub(crate) mod lyrics;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use api_types::{
    common::{ArtistInfo, ErrorResponse, ImageInfo, TagInfo},
    lyrics::{LyricsResponse, UpdateLyricsRequest},
    pagination::{PagedResponse, PaginationParams},
    songs::{CreateSongRequest, SongResponse, SongSummary, SongTagAssignment, UpdateSongRequest},
    tags::SongTagKind,
};
use db::{
    MySqlPool,
    error::DbError,
    models::{NewLyrics, NewSong, UpdateSong},
    queries,
};

use crate::{
    auth::middleware::AuthUser, capabilities, error::ApiError, pagination, state::AppState,
};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_songs,
        get_song,
        create_song,
        update_song,
        delete_song,
        lyrics::get_song_lyrics,
        lyrics::put_song_lyrics,
        lyrics::delete_song_lyrics,
    ),
    components(schemas(
        SongSummary,
        SongResponse,
        CreateSongRequest,
        UpdateSongRequest,
        SongTagAssignment,
        SongTagKind,
        LyricsResponse,
        UpdateLyricsRequest,
        ArtistInfo,
        TagInfo,
        ImageInfo,
        ErrorResponse,
        PagedResponse<SongSummary>,
    ))
)]
pub(crate) struct SongsApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_songs).post(create_song))
        .route("/{id}", get(get_song).put(update_song).delete(delete_song))
        .route(
            "/{id}/lyrics",
            get(lyrics::get_song_lyrics)
                .put(lyrics::put_song_lyrics)
                .delete(lyrics::delete_song_lyrics),
        )
}

/// Loads all related entities for a song row into a full [`SongResponse`].
async fn hydrate(pool: &MySqlPool, song: db::models::Song) -> Result<SongResponse, ApiError> {
    let (artists, tags, images) = tokio::try_join!(
        queries::songs::get_original_artists(pool, song.id),
        queries::songs::get_tags(pool, song.id),
        queries::songs::get_images(pool, song.id),
    )?;

    let artists = artists
        .into_iter()
        .map(|a| ArtistInfo {
            id: a.id,
            name: a.name,
            description: a.description,
        })
        .collect();

    let tags = tags
        .into_iter()
        .map(|t| TagInfo {
            id: t.id,
            name: t.name,
            kind: t.kind,
        })
        .collect::<Vec<_>>();

    let images = images
        .into_iter()
        .map(|i| ImageInfo {
            id: i.id,
            public_url: i.public_url,
            credits: i.credits,
        })
        .collect();

    Ok(SongResponse {
        id: song.id,
        title: song.title,
        date_added: song.date_added,
        artists,
        tags,
        images,
    })
}

fn tag_pairs(assignments: &[SongTagAssignment]) -> Vec<(Uuid, &str)> {
    assignments
        .iter()
        .map(|a| (a.tag_id, a.kind.as_str()))
        .collect()
}

#[utoipa::path(
    get,
    path = "/api/songs",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paged list of songs", body = PagedResponse<SongSummary>),
    ),
    tag = "songs"
)]
pub(crate) async fn list_songs(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PagedResponse<SongSummary>>, ApiError> {
    let (limit, offset) = pagination::limit_offset(&params);

    let (total, songs) = tokio::try_join!(
        queries::songs::count(&state.pool),
        queries::songs::list(&state.pool, limit, offset),
    )?;

    let song_ids: Vec<Uuid> = songs.iter().map(|s| s.id).collect();
    let mut artists_by_song =
        queries::songs::get_original_artists_batch(&state.pool, &song_ids).await?;

    let items = songs
        .into_iter()
        .map(|s| {
            let artists = artists_by_song
                .remove(&s.id)
                .unwrap_or_default()
                .into_iter()
                .map(|a| ArtistInfo {
                    id: a.id,
                    name: a.name,
                    description: a.description,
                })
                .collect();
            SongSummary {
                id: s.id,
                title: s.title,
                date_added: s.date_added,
                artists,
            }
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
    get,
    path = "/api/songs/{id}",
    params(("id" = Uuid, Path, description = "Song ID")),
    responses(
        (status = 200, description = "Song detail", body = SongResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "songs"
)]
pub(crate) async fn get_song(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SongResponse>, ApiError> {
    let song = queries::songs::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(hydrate(&state.pool, song).await?))
}

#[utoipa::path(
    post,
    path = "/api/songs",
    request_body = CreateSongRequest,
    responses(
        (status = 201, description = "Created song", body = SongResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "songs"
)]
pub(crate) async fn create_song(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateSongRequest>,
) -> Result<(StatusCode, Json<SongResponse>), ApiError> {
    if !auth.capabilities.contains(capabilities::SONGS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;

    let lyrics_id = match req.lyrics {
        Some(content) => {
            let l = queries::lyrics::create(&mut tx, &NewLyrics { content }).await?;
            Some(l.id)
        }
        None => None,
    };

    let song = queries::songs::create(
        &mut tx,
        &NewSong {
            title: req.title,
            created_by: None,
            lyrics_id,
        },
    )
    .await?;

    let tag_pairs = tag_pairs(&req.tags);
    queries::songs::set_original_artists(&mut tx, song.id, &req.artist_ids).await?;
    queries::songs::set_tags(&mut tx, song.id, &tag_pairs).await?;
    queries::songs::set_images(&mut tx, song.id, &req.image_ids).await?;

    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok((StatusCode::CREATED, Json(hydrate(&state.pool, song).await?)))
}

#[utoipa::path(
    put,
    path = "/api/songs/{id}",
    params(("id" = Uuid, Path, description = "Song ID")),
    request_body = UpdateSongRequest,
    responses(
        (status = 200, description = "Updated song", body = SongResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "songs"
)]
pub(crate) async fn update_song(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSongRequest>,
) -> Result<Json<SongResponse>, ApiError> {
    if !auth.capabilities.contains(capabilities::SONGS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;

    let song = queries::songs::update(&mut tx, id, &UpdateSong { title: req.title })
        .await?
        .ok_or(ApiError::NotFound)?;

    let tag_pairs = tag_pairs(&req.tags);
    queries::songs::set_original_artists(&mut tx, id, &req.artist_ids).await?;
    queries::songs::set_tags(&mut tx, id, &tag_pairs).await?;
    queries::songs::set_images(&mut tx, id, &req.image_ids).await?;

    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok(Json(hydrate(&state.pool, song).await?))
}

#[utoipa::path(
    delete,
    path = "/api/songs/{id}",
    params(("id" = Uuid, Path, description = "Song ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "songs"
)]
pub(crate) async fn delete_song(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !auth.capabilities.contains(capabilities::SONGS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let found = queries::songs::delete(&state.pool, id).await?;
    if found {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}
