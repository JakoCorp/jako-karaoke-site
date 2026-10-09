//! Video subresource handlers for performances.

use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
};
use sha2::{Digest, Sha256};
use tracing::error;
use uuid::Uuid;

use api_types::{
    common::ErrorResponse,
    performances::{AddVideoLinkRequest, UpdateVideoKindRequest, VideoInfo, VideoKind},
};
use db::{
    error::DbError,
    models::{NewExternalAsset, NewInternalAsset, NewPerformanceVideo},
    queries,
};

use crate::{
    assets, auth::middleware::AuthUser, capabilities, convert, error::ApiError, media,
    state::AppState,
};

/// Placeholder schema for video multipart upload bodies.
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
pub(crate) struct VideoUpload {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
    /// Semantic role. See [`VideoKind`].
    pub kind: String,
    pub title: Option<String>,
    pub credits: Option<String>,
    pub source_url: Option<String>,
}

struct VideoFields {
    data: Vec<u8>,
    content_type: String,
    filename: Option<String>,
    kind: String,
    title: Option<String>,
    credits: Option<String>,
    source_url: Option<String>,
}

async fn read_video_fields(multipart: &mut Multipart) -> Result<VideoFields, ApiError> {
    let mut data: Option<Vec<u8>> = None;
    let mut content_type = String::new();
    let mut filename: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut title: Option<String> = None;
    let mut credits: Option<String> = None;
    let mut source_url: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("multipart field error: {e:?}");
        ApiError::BadRequest(e.to_string())
    })? {
        match field.name() {
            Some("file") => {
                content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                filename = field.file_name().map(str::to_string);
                let bytes = field.bytes().await.map_err(|e| {
                    error!("multipart read error: {e:?}");
                    ApiError::BadRequest(e.to_string())
                })?;
                data = Some(bytes.to_vec());
            }
            Some("kind") => {
                kind = Some(field.text().await.map_err(|e| {
                    error!("multipart field error: {e:?}");
                    ApiError::BadRequest(e.to_string())
                })?);
            }
            Some("title") => {
                let text = field.text().await.map_err(|e| {
                    error!("multipart field error: {e:?}");
                    ApiError::BadRequest(e.to_string())
                })?;
                if !text.is_empty() {
                    title = Some(text);
                }
            }
            Some("credits") => {
                let text = field.text().await.map_err(|e| {
                    error!("multipart field error: {e:?}");
                    ApiError::BadRequest(e.to_string())
                })?;
                if !text.is_empty() {
                    credits = Some(text);
                }
            }
            Some("source_url") => {
                let text = field.text().await.map_err(|e| {
                    error!("multipart field error: {e:?}");
                    ApiError::BadRequest(e.to_string())
                })?;
                if !text.is_empty() {
                    source_url = Some(text);
                }
            }
            _ => {}
        }
    }

    Ok(VideoFields {
        data: data.ok_or_else(|| ApiError::BadRequest("missing 'file' field".into()))?,
        content_type,
        filename,
        kind: kind.ok_or_else(|| ApiError::BadRequest("missing 'kind' field".into()))?,
        title,
        credits,
        source_url,
    })
}

fn validate_video_kind(kind: &str) -> Result<&'static str, ApiError> {
    match kind.trim() {
        "clip" => Ok(VideoKind::Clip.as_str()),
        "vod" => Ok(VideoKind::Vod.as_str()),
        "misc" => Ok(VideoKind::Misc.as_str()),
        other => Err(ApiError::BadRequest(format!(
            "invalid video kind '{other}'"
        ))),
    }
}

#[utoipa::path(
    post,
    path = "/api/performances/{id}/video",
    params(("id" = Uuid, Path, description = "Performance ID")),
    request_body(content = VideoUpload, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Video uploaded and linked", body = VideoInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn upload_video(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<VideoInfo>), ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let fields = read_video_fields(&mut multipart).await?;
    let kind = validate_video_kind(&fields.kind)?;
    let title = fields.title.or_else(|| fields.filename.clone());
    let ext = media::resolve_ext(
        media::MediaKind::Video,
        &fields.content_type,
        fields.filename.as_deref(),
    )?;
    let hash = hex::encode(Sha256::digest(&fields.data));

    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let asset = if let Some(existing) = queries::assets::get_by_hash(&state.pool, &hash).await? {
        existing
    } else {
        let saved = state.store.save("video", ext, &fields.data).await?;
        queries::assets::create_internal(
            &mut conn,
            &NewInternalAsset {
                title,
                credits: fields.credits,
                source_url: fields.source_url,
                hash,
                storage_url: saved.storage_url,
                internal_path: Some(saved.internal_path),
            },
        )
        .await?
    };

    let row = queries::performance_videos::link(
        &mut conn,
        id,
        &NewPerformanceVideo {
            asset_id: asset.id,
            kind: kind.to_string(),
        },
    )
    .await?;

    Ok((StatusCode::CREATED, Json(convert::video_info(row))))
}

#[utoipa::path(
    post,
    path = "/api/performances/{id}/video/link",
    params(("id" = Uuid, Path, description = "Performance ID")),
    request_body = AddVideoLinkRequest,
    responses(
        (status = 201, description = "Video link created", body = VideoInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn add_video_link(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<AddVideoLinkRequest>,
) -> Result<(StatusCode, Json<VideoInfo>), ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let kind = validate_video_kind(&body.kind)?;
    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let asset = queries::assets::create_external(
        &mut conn,
        &NewExternalAsset {
            title: body.title,
            credits: body.credits,
            source_url: body.source_url,
            external_url: body.external_url,
        },
    )
    .await?;

    let row = queries::performance_videos::link(
        &mut conn,
        id,
        &NewPerformanceVideo {
            asset_id: asset.id,
            kind: kind.to_string(),
        },
    )
    .await?;

    Ok((StatusCode::CREATED, Json(convert::video_info(row))))
}

#[utoipa::path(
    patch,
    path = "/api/performances/{id}/video/{asset_id}",
    params(
        ("id" = Uuid, Path, description = "Performance ID"),
        ("asset_id" = Uuid, Path, description = "Video asset ID"),
    ),
    request_body = UpdateVideoKindRequest,
    responses(
        (status = 200, description = "Kind updated", body = VideoInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn update_video_kind(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, asset_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateVideoKindRequest>,
) -> Result<Json<VideoInfo>, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }

    let kind = validate_video_kind(&body.kind)?;
    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let updated = queries::performance_videos::update_kind(&mut *conn, id, asset_id, kind).await?;
    if !updated {
        return Err(ApiError::NotFound);
    }

    let rows = queries::performance_videos::list_for_performance(&state.pool, id).await?;
    let row = rows
        .into_iter()
        .find(|r| r.asset_id == asset_id)
        .ok_or(ApiError::NotFound)?;

    Ok(Json(convert::video_info(row)))
}

#[utoipa::path(
    delete,
    path = "/api/performances/{id}/video/{asset_id}",
    params(
        ("id" = Uuid, Path, description = "Performance ID"),
        ("asset_id" = Uuid, Path, description = "Video asset ID"),
    ),
    responses(
        (status = 204, description = "Unlinked"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn delete_video(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, asset_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }

    let removed = queries::performance_videos::unlink(&state.pool, id, asset_id).await?;
    if !removed {
        return Err(ApiError::NotFound);
    }
    assets::delete_if_unreferenced(&state, asset_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
