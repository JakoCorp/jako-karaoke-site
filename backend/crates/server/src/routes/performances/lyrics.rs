//! Lyrics subresource handlers for performances: `GET/PUT/DELETE /api/performances/{id}/lyrics`.
//!
//! GET falls back to the first linked song's lyrics when the performance has no
//! lyrics override set.
//!
//! PUT creates or updates a performance specific lyrics override.
//!
//! DELETE removes the override and reverts to song fallback. The underlying
//! lyrics row is deleted only if no other songs or performances still reference it.

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

    path = "/api/performances/{id}/lyrics",
    params(("id" = Uuid, Path, description = "Performance ID")),
    responses(
        (status = 200, description = "Lyrics for this performance. Returns performance-specific \
                                      lyrics if set, otherwise falls back to the linked song's \
                                      lyrics.", body = LyricsResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn get_performance_lyrics(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LyricsResponse>, ApiError> {
    let perf = queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if let Some(lyrics_id) = perf.lyrics_id {
        let lyrics = queries::lyrics::get_by_id(&state.pool, lyrics_id)
            .await?
            .ok_or(ApiError::NotFound)?;
        return Ok(Json(LyricsResponse {
            content: lyrics.content,
        }));
    }

    let content = queries::performances::get_fallback_song_lyrics(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(LyricsResponse { content }))
}

#[utoipa::path(
    put,

    path = "/api/performances/{id}/lyrics",
    params(("id" = Uuid, Path, description = "Performance ID")),
    request_body = UpdateLyricsRequest,
    responses(
        (status = 204, description = "Lyrics saved"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn put_performance_lyrics(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLyricsRequest>,
) -> Result<StatusCode, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    let perf = queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    match perf.lyrics_id {
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
            queries::performances::update_lyrics_id(&mut *tx, id, Some(lyrics.id)).await?;
            tx.commit().await.map_err(DbError::Sqlx)?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,

    path = "/api/performances/{id}/lyrics",
    params(("id" = Uuid, Path, description = "Performance ID")),
    responses(
        (status = 204, description = "Performance lyrics override removed"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn delete_performance_lyrics(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    let perf = queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let Some(lyrics_id) = perf.lyrics_id else {
        return Err(ApiError::NotFound);
    };

    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;
    queries::performances::update_lyrics_id(&mut *tx, id, None).await?;
    let refs = queries::lyrics::reference_count(&mut *tx, lyrics_id).await?;
    if refs == 0 {
        queries::lyrics::delete(&mut *tx, lyrics_id).await?;
    }
    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok(StatusCode::NO_CONTENT)
}
