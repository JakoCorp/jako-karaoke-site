//! Play session handlers for performances.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use api_types::{
    common::ErrorResponse,
    pagination::{PagedResponse, defaults as pagination_defaults},
    plays::{CompletePlayResponse, PerformancePlayer, StartPlayResponse},
    users::UserSummary,
};
use db::{error::DbError, queries};

use crate::{
    auth::middleware::AuthUser, capabilities, error::ApiError, pagination, plays, state::AppState,
};

/// Query parameters for `GET /api/performances/{id}/plays`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct PerformancePlayerListParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "pagination_defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "pagination_defaults::per_page")]
    pub per_page: u32,
}

#[utoipa::path(
    get,
    path = "/api/performances/{id}/plays",
    params(
        ("id" = Uuid, Path, description = "Performance ID"),
        PerformancePlayerListParams,
    ),
    responses(
        (status = 200, description = "Users who played this performance, most plays first", body = PagedResponse<PerformancePlayer>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn list_performance_plays(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(params): Query<PerformancePlayerListParams>,
) -> Result<Json<PagedResponse<PerformancePlayer>>, ApiError> {
    if !auth.capabilities.contains(capabilities::PLAYS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let (limit, offset) = pagination::limit_offset(params.page, params.per_page);
    let (rows, total) = tokio::try_join!(
        queries::performance_plays::list_players_for_performance(&state.pool, id, limit, offset),
        queries::performance_plays::count_players_for_performance(&state.pool, id),
    )?;

    let items = rows
        .into_iter()
        .map(|row| PerformancePlayer {
            user: UserSummary {
                id: row.user_id,
                username: row.username,
            },
            play_count: row.play_count,
            last_played_at: row.last_played_at,
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
    post,
    path = "/api/performances/{id}/plays",
    params(("id" = Uuid, Path, description = "Performance ID")),
    responses(
        (status = 201, description = "Play session opened", body = StartPlayResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn start_play(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<StartPlayResponse>), ApiError> {
    let performance = queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;
    queries::performance_plays::delete_pending_for_user(&mut *tx, auth.user_id).await?;
    let play = queries::performance_plays::create(&mut tx, id, auth.user_id).await?;
    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok((
        StatusCode::CREATED,
        Json(StartPlayResponse {
            play_id: play.id,
            required_seconds: plays::required_seconds(performance.duration),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/performances/{id}/plays/{play_id}/complete",
    params(
        ("id" = Uuid, Path, description = "Performance ID"),
        ("play_id" = Uuid, Path, description = "Play session ID"),
    ),
    responses(
        (status = 200, description = "Play session completed", body = CompletePlayResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Play session not found", body = ErrorResponse),
        (status = 409, description = "Not enough listening time, or the session expired", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn complete_play(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, play_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CompletePlayResponse>, ApiError> {
    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;

    let progress =
        queries::performance_plays::get_progress_for_update(&mut tx, play_id, auth.user_id)
            .await?
            .filter(|progress| progress.performance_id == id)
            .ok_or(ApiError::NotFound)?;
    if progress.counted {
        return Ok(Json(CompletePlayResponse { counted: true }));
    }

    let performance = queries::performances::get_by_id(&mut *tx, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if progress.elapsed_seconds < i64::from(plays::required_seconds(performance.duration)) {
        return Err(ApiError::Conflict("not enough listening time".to_string()));
    }
    if progress.elapsed_seconds > plays::MAX_SESSION_AGE_SECONDS {
        return Err(ApiError::Conflict("play session expired".to_string()));
    }

    let since_last_counted =
        queries::performance_plays::seconds_since_last_counted(&mut *tx, auth.user_id, id).await?;
    let cooling_down = since_last_counted
        .is_some_and(|seconds| seconds < i64::from(plays::cooldown_seconds(performance.duration)));
    if cooling_down {
        queries::performance_plays::delete(&mut *tx, play_id).await?;
    } else {
        queries::performance_plays::mark_counted(&mut *tx, play_id).await?;
    }
    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok(Json(CompletePlayResponse {
        counted: !cooling_down,
    }))
}
