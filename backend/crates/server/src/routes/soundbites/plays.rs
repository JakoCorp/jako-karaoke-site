//! Play session handlers for soundbites.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use api_types::{
    common::ErrorResponse,
    pagination::{PagedResponse, defaults as pagination_defaults},
    plays::{CompletePlayResponse, SoundbitePlayer, StartPlayResponse},
};
use db::{error::DbError, queries};

use crate::{
    auth::middleware::AuthUser,
    capabilities, convert,
    error::ApiError,
    pagination,
    plays::{self, soundbite as policy},
    state::AppState,
};

/// Query parameters for `GET /api/soundbites/{id}/plays`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct SoundbitePlayerListParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "pagination_defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "pagination_defaults::per_page")]
    pub per_page: u32,
}

#[utoipa::path(
    get,
    path = "/api/soundbites/{id}/plays",
    params(
        ("id" = Uuid, Path, description = "Soundbite ID"),
        SoundbitePlayerListParams,
    ),
    responses(
        (status = 200, description = "Users who played this soundbite, most plays first", body = PagedResponse<SoundbitePlayer>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Soundbite not found", body = ErrorResponse),
    ),
    tag = "soundbites",
    security(("session" = []))
)]
pub(crate) async fn list_soundbite_plays(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(params): Query<SoundbitePlayerListParams>,
) -> Result<Json<PagedResponse<SoundbitePlayer>>, ApiError> {
    if !auth.capabilities.contains(capabilities::PLAYS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    queries::soundbites::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let (limit, offset) = pagination::limit_offset(params.page, params.per_page);
    let (rows, total) = tokio::try_join!(
        queries::soundbite_plays::list_players_for_soundbite(&state.pool, id, limit, offset),
        queries::soundbite_plays::count_players_for_soundbite(&state.pool, id),
    )?;

    Ok(Json(PagedResponse {
        items: rows.into_iter().map(convert::soundbite_player).collect(),
        total,
        page: params.page,
        per_page: limit,
    }))
}

#[utoipa::path(
    post,
    path = "/api/soundbites/{id}/plays",
    params(("id" = Uuid, Path, description = "Soundbite ID")),
    responses(
        (status = 201, description = "Play session opened", body = StartPlayResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Soundbite not found", body = ErrorResponse),
    ),
    tag = "soundbites",
    security(("session" = []))
)]
pub(crate) async fn start_soundbite_play(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<StartPlayResponse>), ApiError> {
    queries::soundbites::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let length = queries::soundbites::audio_length(&state.pool, id).await?;

    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;
    queries::soundbite_plays::delete_pending_for_user(&mut *tx, auth.user_id).await?;
    let play = queries::soundbite_plays::create(&mut tx, id, auth.user_id).await?;
    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok((
        StatusCode::CREATED,
        Json(StartPlayResponse {
            play_id: play.id,
            required_seconds: policy::required_seconds(length),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/soundbites/{id}/plays/{play_id}/complete",
    params(
        ("id" = Uuid, Path, description = "Soundbite ID"),
        ("play_id" = Uuid, Path, description = "Play session ID"),
    ),
    responses(
        (status = 200, description = "Play session completed", body = CompletePlayResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Play session not found", body = ErrorResponse),
        (status = 409, description = "Not enough listening time, or the session expired", body = ErrorResponse),
    ),
    tag = "soundbites",
    security(("session" = []))
)]
pub(crate) async fn complete_soundbite_play(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, play_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CompletePlayResponse>, ApiError> {
    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;

    let progress =
        queries::soundbite_plays::get_progress_for_update(&mut tx, play_id, auth.user_id)
            .await?
            .filter(|progress| progress.soundbite_id == id)
            .ok_or(ApiError::NotFound)?;
    if progress.counted {
        return Ok(Json(CompletePlayResponse { counted: true }));
    }

    let length = queries::soundbites::audio_length(&mut *tx, id).await?;

    if !policy::is_long_enough(progress.elapsed_seconds, length) {
        return Err(ApiError::Conflict("not enough listening time".to_string()));
    }
    if progress.elapsed_seconds > plays::MAX_SESSION_AGE_SECONDS {
        return Err(ApiError::Conflict("play session expired".to_string()));
    }

    let since_last_counted =
        queries::soundbite_plays::seconds_since_last_counted(&mut *tx, auth.user_id, id).await?;
    let cooling_down = since_last_counted
        .is_some_and(|seconds| seconds < i64::from(policy::cooldown_seconds(length)));
    if cooling_down {
        queries::soundbite_plays::delete(&mut *tx, play_id).await?;
    } else {
        queries::soundbite_plays::mark_counted(&mut *tx, play_id).await?;
    }
    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok(Json(CompletePlayResponse {
        counted: !cooling_down,
    }))
}
