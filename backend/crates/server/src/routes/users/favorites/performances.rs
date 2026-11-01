//! Performance favorites handlers for users.

use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use api_types::{common::ErrorResponse, playlists::PlaylistEntry};
use db::queries;

use crate::{
    auth::middleware::AuthUser, convert, error::ApiError, routes::users::can_view_private,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/users/{id}/favorites/performances",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "Ordered performances in this user's favorites playlist.", body = Vec<PlaylistEntry>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
pub(crate) async fn list_user_favorite_performances(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: Option<AuthUser>,
) -> Result<Json<Vec<PlaylistEntry>>, ApiError> {
    if !can_view_private(&auth, id) {
        return Err(ApiError::Forbidden);
    }

    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let playlist = queries::playlists::get_favorites_by_user(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let rows = queries::playlists::get_performances_in_playlist(&state.pool, playlist.id).await?;
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
