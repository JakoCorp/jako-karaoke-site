//! Lyrics subresource handlers for songs: `GET/PUT/DELETE /api/songs/{id}/lyrics`.
//!
//! PUT creates the lyrics row if none exists, or updates the content in place
//! if one already exists (shared ownership semantics).
//!
//! DELETE unlinks the FK and deletes the lyrics row only if no other songs or
//! performances still reference it.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use api_types::{
    common::ErrorResponse,
    lyrics::{LyricsResponse, UpdateLyricsRequest},
};
use db::{error::DbError, models::NewLyrics, queries};

use crate::{auth::middleware::AuthUser, capabilities, error::ApiError, state::AppState};

#[utoipa::path(
    get,

    path = "/api/songs/{id}/lyrics",
    params(("id" = Uuid, Path, description = "Song ID")),
    responses(
        (status = 200, description = "Song lyrics", body = LyricsResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "songs"
)]
pub(crate) async fn get_song_lyrics(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LyricsResponse>, ApiError> {
    let song = queries::songs::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let lyrics_id = song.lyrics_id.ok_or(ApiError::NotFound)?;
    let lyrics = queries::lyrics::get_by_id(&state.pool, lyrics_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(LyricsResponse {
        content: lyrics.content,
    }))
}

#[utoipa::path(
    put,

    path = "/api/songs/{id}/lyrics",
    params(("id" = Uuid, Path, description = "Song ID")),
    request_body = UpdateLyricsRequest,
    responses(
        (status = 204, description = "Lyrics saved"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
    ),
    tag = "songs"
)]
pub(crate) async fn put_song_lyrics(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLyricsRequest>,
) -> Result<StatusCode, ApiError> {
    if !auth.capabilities.contains(capabilities::SONGS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let song = queries::songs::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    match song.lyrics_id {
        Some(existing_id) => {
            queries::lyrics::update(&state.pool, existing_id, &req.content).await?;
        }
        None => {
            let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;
            let lyrics = queries::lyrics::create(
                &mut tx,
                &NewLyrics {
                    content: req.content,
                },
            )
            .await?;
            queries::songs::update_lyrics_id(&mut *tx, id, Some(lyrics.id)).await?;
            tx.commit().await.map_err(DbError::Sqlx)?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,

    path = "/api/songs/{id}/lyrics",
    params(("id" = Uuid, Path, description = "Song ID")),
    responses(
        (status = 204, description = "Lyrics removed"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "songs"
)]
pub(crate) async fn delete_song_lyrics(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !auth.capabilities.contains(capabilities::SONGS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let song = queries::songs::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let Some(lyrics_id) = song.lyrics_id else {
        return Err(ApiError::NotFound);
    };

    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;
    queries::songs::update_lyrics_id(&mut *tx, id, None).await?;
    let refs = queries::lyrics::reference_count(&mut *tx, lyrics_id).await?;
    if refs == 0 {
        queries::lyrics::delete(&mut *tx, lyrics_id).await?;
    }
    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok(StatusCode::NO_CONTENT)
}
