//! Soundbite play moderation handlers for users.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use api_types::{common::ErrorResponse, pagination::PagedResponse, plays::UserSoundbitePlayEntry};
use db::queries;

use super::UserPlayListParams;
use crate::{
    auth::middleware::AuthUser, capabilities, convert, error::ApiError, pagination, state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/users/{id}/plays/soundbites",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        UserPlayListParams,
    ),
    responses(
        (status = 200, description = "Soundbites this user has counted plays on, most recent first", body = PagedResponse<UserSoundbitePlayEntry>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn list_user_soundbite_plays(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(params): Query<UserPlayListParams>,
) -> Result<Json<PagedResponse<UserSoundbitePlayEntry>>, ApiError> {
    if !auth.capabilities.contains(capabilities::PLAYS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let (limit, offset) = pagination::limit_offset(params.page, params.per_page);
    let (rows, total) = tokio::try_join!(
        queries::soundbite_plays::list_for_user(&state.pool, id, limit, offset),
        queries::soundbite_plays::count_for_user(&state.pool, id),
    )?;

    Ok(Json(PagedResponse {
        items: rows
            .into_iter()
            .map(convert::user_soundbite_play_entry)
            .collect(),
        total,
        page: params.page,
        per_page: limit,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/users/{id}/plays/soundbites",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 204, description = "All counted soundbite plays of this user removed"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn delete_user_soundbite_plays(
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
    queries::soundbite_plays::delete_counted_for_user(&state.pool, id, None).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/users/{id}/plays/soundbites/{soundbite_id}",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        ("soundbite_id" = Uuid, Path, description = "Soundbite ID"),
    ),
    responses(
        (status = 204, description = "Counted plays of this user on the soundbite removed"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn delete_user_plays_on_soundbite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, soundbite_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    if !auth.capabilities.contains(capabilities::PLAYS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    queries::soundbite_plays::delete_counted_for_user(&state.pool, id, Some(soundbite_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
