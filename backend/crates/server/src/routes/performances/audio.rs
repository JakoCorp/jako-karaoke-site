//! Audio subresource handlers for performances.

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
    performances::{AddAudioLinkRequest, AudioInfo, AudioKind, UpdateAudioKindRequest},
};
use db::{
    error::DbError,
    models::{NewExternalAsset, NewInternalAsset, NewPerformanceAudio},
    queries,
};

use crate::{
    auth::middleware::AuthUser, capabilities, convert, error::ApiError, media, state::AppState,
};

/// Placeholder schema for audio multipart upload bodies.
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
pub(crate) struct AudioUpload {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
    /// Semantic role. See [`AudioKind`].
    pub kind: String,
    pub title: Option<String>,
    pub credits: Option<String>,
    pub source_url: Option<String>,
}

struct AudioFields {
    data: Vec<u8>,
    content_type: String,
    filename: Option<String>,
    kind: String,
    title: Option<String>,
    credits: Option<String>,
    source_url: Option<String>,
}

async fn read_audio_fields(multipart: &mut Multipart) -> Result<AudioFields, ApiError> {
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

    Ok(AudioFields {
        data: data.ok_or_else(|| ApiError::BadRequest("missing 'file' field".into()))?,
        content_type,
        filename,
        kind: kind.ok_or_else(|| ApiError::BadRequest("missing 'kind' field".into()))?,
        title,
        credits,
        source_url,
    })
}

fn validate_audio_kind(kind: &str) -> Result<&'static str, ApiError> {
    match kind.trim() {
        "primary" => Ok(AudioKind::Primary.as_str()),
        "misc" => Ok(AudioKind::Misc.as_str()),
        other => Err(ApiError::BadRequest(format!(
            "invalid audio kind '{other}'"
        ))),
    }
}

#[utoipa::path(
    post,
    path = "/api/performances/{id}/audio",
    params(("id" = Uuid, Path, description = "Performance ID")),
    request_body(content = AudioUpload, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Audio uploaded and linked", body = AudioInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn upload_audio(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<AudioInfo>), ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let fields = read_audio_fields(&mut multipart).await?;
    let kind = validate_audio_kind(&fields.kind)?;
    let title = fields.title.or_else(|| fields.filename.clone());
    let ext = media::resolve_ext(
        media::MediaKind::Audio,
        &fields.content_type,
        fields.filename.as_deref(),
    )?;
    let hash = hex::encode(Sha256::digest(&fields.data));

    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let asset = if let Some(existing) = queries::assets::get_by_hash(&state.pool, &hash).await? {
        existing
    } else {
        let saved = state.store.save("audio", ext, &fields.data).await?;
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

    if kind == AudioKind::Primary.as_str() {
        queries::performance_audios::unset_primary(&mut *conn, id).await?;
    }
    let row = queries::performance_audios::link(
        &mut conn,
        id,
        &NewPerformanceAudio {
            asset_id: asset.id,
            kind: kind.to_string(),
        },
    )
    .await?;

    Ok((StatusCode::CREATED, Json(convert::audio_info(row))))
}

#[utoipa::path(
    post,
    path = "/api/performances/{id}/audio/link",
    params(("id" = Uuid, Path, description = "Performance ID")),
    request_body = AddAudioLinkRequest,
    responses(
        (status = 201, description = "Audio link created", body = AudioInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn add_audio_link(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<AddAudioLinkRequest>,
) -> Result<(StatusCode, Json<AudioInfo>), ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let kind = validate_audio_kind(&body.kind)?;
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

    if kind == AudioKind::Primary.as_str() {
        queries::performance_audios::unset_primary(&mut *conn, id).await?;
    }
    let row = queries::performance_audios::link(
        &mut conn,
        id,
        &NewPerformanceAudio {
            asset_id: asset.id,
            kind: kind.to_string(),
        },
    )
    .await?;

    Ok((StatusCode::CREATED, Json(convert::audio_info(row))))
}

#[utoipa::path(
    patch,
    path = "/api/performances/{id}/audio/{asset_id}",
    params(
        ("id" = Uuid, Path, description = "Performance ID"),
        ("asset_id" = Uuid, Path, description = "Audio asset ID"),
    ),
    request_body = UpdateAudioKindRequest,
    responses(
        (status = 200, description = "Kind updated", body = AudioInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn update_audio_kind(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, asset_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateAudioKindRequest>,
) -> Result<Json<AudioInfo>, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }

    let kind = validate_audio_kind(&body.kind)?;
    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    if kind == AudioKind::Primary.as_str() {
        queries::performance_audios::unset_primary(&mut *conn, id).await?;
    }
    let updated = queries::performance_audios::update_kind(&mut *conn, id, asset_id, kind).await?;
    if !updated {
        return Err(ApiError::NotFound);
    }

    let rows = queries::performance_audios::list_for_performance(&state.pool, id).await?;
    let row = rows
        .into_iter()
        .find(|r| r.asset_id == asset_id)
        .ok_or(ApiError::NotFound)?;

    Ok(Json(convert::audio_info(row)))
}

#[utoipa::path(
    delete,
    path = "/api/performances/{id}/audio/{asset_id}",
    params(
        ("id" = Uuid, Path, description = "Performance ID"),
        ("asset_id" = Uuid, Path, description = "Audio asset ID"),
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
pub(crate) async fn delete_audio(
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

    let asset = queries::assets::get_by_id(&state.pool, asset_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let removed = queries::performance_audios::unlink(&state.pool, id, asset_id).await?;
    if !removed {
        return Err(ApiError::NotFound);
    }

    let ref_count = queries::assets::reference_count(&state.pool, asset_id).await?;
    if ref_count == 0 {
        queries::assets::delete(&state.pool, asset_id).await?;
        if let Some(path) = &asset.internal_path
            && let Err(e) = state.store.delete(path).await
        {
            error!("failed to delete audio file {path}: {e}");
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
