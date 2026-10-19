//! Play moderation handlers for users.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use api_types::{
    common::ErrorResponse,
    pagination::{PagedResponse, defaults as pagination_defaults},
    plays::UserPlayEntry,
};
use db::queries;

use crate::{
    auth::middleware::AuthUser, capabilities, convert, error::ApiError, pagination, state::AppState,
};

/// Query parameters for `GET /api/users/{id}/plays`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct PlayedPerformanceListParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "pagination_defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "pagination_defaults::per_page")]
    pub per_page: u32,
}

#[utoipa::path(
    get,
    path = "/api/users/{id}/plays",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        PlayedPerformanceListParams,
    ),
    responses(
        (status = 200, description = "Performances this user has counted plays on, most recent first", body = PagedResponse<UserPlayEntry>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn list_user_plays(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(params): Query<PlayedPerformanceListParams>,
) -> Result<Json<PagedResponse<UserPlayEntry>>, ApiError> {
    if !auth.capabilities.contains(capabilities::PLAYS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let (limit, offset) = pagination::limit_offset(params.page, params.per_page);
    let (rows, total) = tokio::try_join!(
        queries::performance_plays::list_for_user(&state.pool, id, limit, offset),
        queries::performance_plays::count_for_user(&state.pool, id),
    )?;

    let play_stats: Vec<_> = rows
        .iter()
        .map(|row| (row.user_play_count, row.last_played_at))
        .collect();
    let performances: Vec<_> = rows.into_iter().map(|row| row.performance).collect();
    let summaries =
        crate::routes::performances::build_performance_summaries(&state.pool, performances).await?;
    let items = summaries
        .into_iter()
        .zip(play_stats)
        .map(|(summary, (user_play_count, last_played_at))| {
            convert::user_play_entry(summary, user_play_count, last_played_at)
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
    delete,
    path = "/api/users/{id}/plays",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 204, description = "All counted plays of this user removed"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn delete_user_plays(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !auth.capabilities.contains(capabilities::PLAYS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    queries::performance_plays::delete_counted_for_user(&state.pool, id, None).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/users/{id}/plays/{performance_id}",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        ("performance_id" = Uuid, Path, description = "Performance ID"),
    ),
    responses(
        (status = 204, description = "Counted plays of this user on the performance removed"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn delete_user_performance_plays(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, performance_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    if !auth.capabilities.contains(capabilities::PLAYS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    queries::performance_plays::delete_counted_for_user(&state.pool, id, Some(performance_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
