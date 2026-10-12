//! Account management for the authenticated user.

use axum::{
    Json,
    extract::{Multipart, State},
};
use chrono::{DateTime, Duration, Utc};
use tracing::error;

use api_types::{
    auth::{MeResponse, UpdateMeRequest},
    common::ErrorResponse,
};
use db::{error::DbError, models::User, queries};

use super::{avatar, middleware::AuthUser, validate_username};
use crate::{error::ApiError, media, state::AppState};

/// Minimum time between username changes.
const USERNAME_COOLDOWN_DAYS: i64 = 30;

/// Placeholder schema for the avatar multipart upload body.
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
pub(crate) struct AvatarUpload {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}

/// Builds the [`MeResponse`] for a user, resolving their avatar URL and username cooldown.
pub(crate) async fn me_response(
    state: &AppState,
    user: User,
    capabilities: impl IntoIterator<Item = String>,
) -> Result<MeResponse, ApiError> {
    let avatar_url = queries::user_avatars::get_storage_url(&state.pool, user.id).await?;
    Ok(MeResponse {
        id: user.id,
        username_changeable_at: username_changeable_at(&user),
        username: user.username,
        avatar_url,
        capabilities: capabilities.into_iter().collect(),
    })
}

fn username_changeable_at(user: &User) -> Option<DateTime<Utc>> {
    user.username_changed_at
        .map(|changed_at| changed_at + Duration::days(USERNAME_COOLDOWN_DAYS))
        .filter(|changeable_at| *changeable_at > Utc::now())
}

#[utoipa::path(
    patch,
    path = "/auth/me",
    request_body = UpdateMeRequest,
    responses(
        (status = 200, description = "Username updated", body = MeResponse),
        (status = 400, description = "Invalid username or changed too recently", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Username already taken", body = ErrorResponse),
    ),
    tag = "auth",
    security(("session" = []))
)]
pub(crate) async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateMeRequest>,
) -> Result<Json<MeResponse>, ApiError> {
    validate_username(&req.username)?;

    let user = queries::users::get_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if user.username == req.username {
        return Ok(Json(me_response(&state, user, auth.capabilities).await?));
    }

    if let Some(changeable_at) = username_changeable_at(&user) {
        return Err(ApiError::BadRequest(format!(
            "username can be changed again on {}",
            changeable_at.format("%Y-%m-%d")
        )));
    }

    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let updated = queries::users::update_username(&mut conn, user.id, &req.username)
        .await
        .map_err(|e| match e {
            DbError::Conflict => ApiError::Conflict("username already taken".into()),
            other => ApiError::from(other),
        })?;

    Ok(Json(me_response(&state, updated, auth.capabilities).await?))
}

#[utoipa::path(
    put,
    path = "/auth/me/avatar",
    request_body(content = AvatarUpload, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Avatar replaced", body = MeResponse),
        (status = 400, description = "Missing, oversized or unsupported image", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "auth",
    security(("session" = []))
)]
pub(crate) async fn upload_avatar(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<MeResponse>, ApiError> {
    let mut upload: Option<(Vec<u8>, String, Option<String>)> = None;
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("multipart field error: {e:?}");
        ApiError::BadRequest(e.to_string())
    })? {
        if field.name() == Some("file") {
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let filename = field.file_name().map(str::to_string);
            let bytes = field.bytes().await.map_err(|e| {
                error!("multipart read error: {e:?}");
                ApiError::BadRequest(e.to_string())
            })?;
            upload = Some((bytes.to_vec(), content_type, filename));
        }
    }
    let (data, content_type, filename) =
        upload.ok_or_else(|| ApiError::BadRequest("missing 'file' field".into()))?;

    let ext = media::resolve_ext(media::MediaKind::Image, &content_type, filename.as_deref())?;
    avatar::set_user_avatar(&state, auth.user_id, &data, ext, None).await?;

    let user = queries::users::get_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(me_response(&state, user, auth.capabilities).await?))
}
