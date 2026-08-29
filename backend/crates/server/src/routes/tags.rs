//! Tag CRUD handlers and the `TagsApi` OpenAPI spec struct.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use api_types::{
    common::ErrorResponse,
    tags::{CreateTagRequest, TagResponse},
};
use db::{error::DbError, models::NewTag, queries};

use crate::{auth::middleware::AuthUser, capabilities, convert, error::ApiError, state::AppState};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_tags, get_tag, create_tag, delete_tag),
    components(schemas(TagResponse, CreateTagRequest, ErrorResponse))
)]
pub(crate) struct TagsApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tags).post(create_tag))
        .route("/{id}", get(get_tag).delete(delete_tag))
}

#[utoipa::path(
    get,
    path = "/api/tags",
    responses(
        (status = 200, description = "All tags ordered by name", body = Vec<TagResponse>),
    ),
    tag = "tags"
)]
pub(crate) async fn list_tags(
    State(state): State<AppState>,
) -> Result<Json<Vec<TagResponse>>, ApiError> {
    let tags = queries::tags::list(&state.pool).await?;
    Ok(Json(tags.into_iter().map(convert::tag_response).collect()))
}

#[utoipa::path(
    get,
    path = "/api/tags/{id}",
    params(("id" = Uuid, Path, description = "Tag ID")),
    responses(
        (status = 200, description = "Tag detail", body = TagResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "tags"
)]
pub(crate) async fn get_tag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TagResponse>, ApiError> {
    let tag = queries::tags::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(convert::tag_response(tag)))
}

#[utoipa::path(
    post,
    path = "/api/tags",
    request_body = CreateTagRequest,
    responses(
        (status = 200, description = "Tag returned, created if new", body = TagResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "tags",
    security(("session" = []))
)]
pub(crate) async fn create_tag(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<TagResponse>, ApiError> {
    if !auth.capabilities.contains(capabilities::TAGS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let tag = queries::tags::get_or_create(&mut conn, &NewTag { name: req.name }).await?;
    Ok(Json(convert::tag_response(tag)))
}

#[utoipa::path(
    delete,
    path = "/api/tags/{id}",
    params(("id" = Uuid, Path, description = "Tag ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "tags",
    security(("session" = []))
)]
pub(crate) async fn delete_tag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !auth.capabilities.contains(capabilities::TAGS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let found = queries::tags::delete(&state.pool, id).await?;
    if found {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}
