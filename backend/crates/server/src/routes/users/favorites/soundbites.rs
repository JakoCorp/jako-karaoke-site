//! Soundbite favorites handlers for users.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use api_types::{common::ErrorResponse, pagination::PagedResponse, soundbites::SoundbiteResponse};
use db::queries::{self, soundbites::SoundbiteFilter};

use crate::{
    auth::middleware::AuthUser,
    error::ApiError,
    pagination,
    routes::{
        common::search_text,
        soundbites::{build_soundbite_responses, params::SoundbiteListParams},
    },
    state::AppState,
};

fn require_owner(auth: &AuthUser, user_id: Uuid) -> Result<(), ApiError> {
    if auth.user_id == user_id {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

#[utoipa::path(
    get,
    path = "/api/users/{id}/favorites/soundbites",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        SoundbiteListParams,
    ),
    responses(
        (status = 200, description = "Paged list of the soundbites this user marked as favorite", body = PagedResponse<SoundbiteResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn list_user_favorite_soundbites(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(params): Query<SoundbiteListParams>,
) -> Result<Json<PagedResponse<SoundbiteResponse>>, ApiError> {
    require_owner(&auth, id)?;

    let (limit, offset) = pagination::limit_offset(params.page, params.per_page);
    let filter = SoundbiteFilter {
        q: search_text(params.q.as_deref()),
        favorited_by: Some(id),
    };

    let (sort, direction) = params.order();
    let (total, soundbites) = tokio::try_join!(
        queries::soundbites::search_count(&state.pool, filter),
        queries::soundbites::search(&state.pool, filter, sort, direction, limit, offset),
    )?;

    let items = build_soundbite_responses(&state.pool, soundbites).await?;

    Ok(Json(PagedResponse {
        items,
        total,
        page: params.page,
        per_page: limit,
    }))
}

#[utoipa::path(
    get,
    path = "/api/users/{id}/favorites/soundbites/ids",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "IDs of every soundbite this user marked as favorite", body = Vec<Uuid>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn list_user_favorite_soundbite_ids(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Uuid>>, ApiError> {
    require_owner(&auth, id)?;
    Ok(Json(
        queries::soundbite_favorites::list_ids(&state.pool, id).await?,
    ))
}

#[utoipa::path(
    put,
    path = "/api/users/{id}/favorites/soundbites/{soundbite_id}",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        ("soundbite_id" = Uuid, Path, description = "Soundbite ID"),
    ),
    responses(
        (status = 204, description = "Soundbite is a favorite of this user"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Soundbite not found", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn add_user_favorite_soundbite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, soundbite_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    require_owner(&auth, id)?;
    queries::soundbites::get_by_id(&state.pool, soundbite_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    queries::soundbite_favorites::add(&state.pool, id, soundbite_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/users/{id}/favorites/soundbites/{soundbite_id}",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        ("soundbite_id" = Uuid, Path, description = "Soundbite ID"),
    ),
    responses(
        (status = 204, description = "Soundbite is not a favorite of this user"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "users",
    security(("session" = []))
)]
pub(crate) async fn remove_user_favorite_soundbite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, soundbite_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    require_owner(&auth, id)?;
    queries::soundbite_favorites::remove(&state.pool, id, soundbite_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
