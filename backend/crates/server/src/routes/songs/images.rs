//! Image upload and delete sub-resource for songs.

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
    songs::{AddSongImageLinkRequest, SongImageInfo, SongImageKind, UpdateSongImageRequest},
};
use db::{
    error::DbError,
    models::{NewExternalAsset, NewInternalAsset},
    queries,
};

use crate::{
    assets, auth::middleware::AuthUser, capabilities, convert, error::ApiError, media,
    state::AppState,
};

/// Placeholder schema for image multipart upload bodies.
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
pub(crate) struct ImageUpload {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
    /// Semantic role of the image. See [`SongImageKind`](api_types::songs::SongImageKind).
    pub kind: String,
    pub title: Option<String>,
    pub credits: Option<String>,
    pub source_url: Option<String>,
}

struct ImageFields {
    data: Vec<u8>,
    content_type: String,
    filename: Option<String>,
    kind: String,
    title: Option<String>,
    credits: Option<String>,
    source_url: Option<String>,
}

async fn read_image_fields(multipart: &mut Multipart) -> Result<ImageFields, ApiError> {
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
                let text = field.text().await.map_err(|e| {
                    error!("multipart field error: {e:?}");
                    ApiError::BadRequest(e.to_string())
                })?;
                kind = Some(text);
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

    Ok(ImageFields {
        data: data.ok_or_else(|| ApiError::BadRequest("missing 'file' field".into()))?,
        content_type,
        filename,
        kind: kind.ok_or_else(|| ApiError::BadRequest("missing 'kind' field".into()))?,
        title,
        credits,
        source_url,
    })
}

#[utoipa::path(
    post,
    path = "/api/songs/{id}/images",
    params(("id" = Uuid, Path, description = "Song ID")),
    request_body(content = ImageUpload, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Image uploaded and linked", body = SongImageInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
    ),
    tag = "songs",
    security(("session" = []))
)]
pub(crate) async fn upload_song_image(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<SongImageInfo>), ApiError> {
    if !auth.capabilities.contains(capabilities::SONGS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    queries::songs::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let fields = read_image_fields(&mut multipart).await?;

    let kind: SongImageKind = fields
        .kind
        .trim()
        .parse()
        .map_err(|_| ApiError::BadRequest(format!("invalid kind '{}'", fields.kind.trim())))?;

    let ext = media::resolve_ext(
        media::MediaKind::Image,
        &fields.content_type,
        fields.filename.as_deref(),
    )?;
    let title = fields.title.or_else(|| fields.filename.clone());
    let hash = hex::encode(Sha256::digest(&fields.data));

    let asset = if let Some(existing) = queries::assets::get_by_hash(&state.pool, &hash).await? {
        existing
    } else {
        let saved = state.store.save("images", ext, &fields.data).await?;
        let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
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

    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    queries::songs::link_image(&mut conn, id, asset.id, kind.as_str()).await?;

    let rows = queries::songs::get_images(&state.pool, id).await?;
    let row = rows
        .into_iter()
        .find(|r| r.asset_id == asset.id)
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::CREATED, Json(convert::song_image_info(row))))
}

#[utoipa::path(
    post,
    path = "/api/songs/{id}/images/link",
    params(("id" = Uuid, Path, description = "Song ID")),
    request_body = AddSongImageLinkRequest,
    responses(
        (status = 201, description = "Image link created", body = SongImageInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Song not found", body = ErrorResponse),
    ),
    tag = "songs",
    security(("session" = []))
)]
pub(crate) async fn link_song_image(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<AddSongImageLinkRequest>,
) -> Result<(StatusCode, Json<SongImageInfo>), ApiError> {
    if !auth.capabilities.contains(capabilities::SONGS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    queries::songs::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let kind: SongImageKind = body
        .kind
        .trim()
        .parse()
        .map_err(|_| ApiError::BadRequest(format!("invalid kind '{}'", body.kind.trim())))?;

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

    queries::songs::link_image(&mut conn, id, asset.id, kind.as_str()).await?;

    let rows = queries::songs::get_images(&state.pool, id).await?;
    let row = rows
        .into_iter()
        .find(|r| r.asset_id == asset.id)
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::CREATED, Json(convert::song_image_info(row))))
}

#[utoipa::path(
    patch,
    path = "/api/songs/{id}/images/{asset_id}",
    params(
        ("id" = Uuid, Path, description = "Song ID"),
        ("asset_id" = Uuid, Path, description = "Asset ID"),
    ),
    request_body = UpdateSongImageRequest,
    responses(
        (status = 200, description = "Kind updated", body = SongImageInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "songs",
    security(("session" = []))
)]
pub(crate) async fn update_song_image_kind(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, asset_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateSongImageRequest>,
) -> Result<Json<SongImageInfo>, ApiError> {
    if !auth.capabilities.contains(capabilities::SONGS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }

    let kind: SongImageKind = body
        .kind
        .trim()
        .parse()
        .map_err(|_| ApiError::BadRequest(format!("invalid kind '{}'", body.kind.trim())))?;

    let updated =
        queries::songs::update_image_kind(&state.pool, id, asset_id, kind.as_str()).await?;
    if !updated {
        return Err(ApiError::NotFound);
    }

    let rows = queries::songs::get_images(&state.pool, id).await?;
    let row = rows
        .into_iter()
        .find(|r| r.asset_id == asset_id)
        .ok_or(ApiError::NotFound)?;

    Ok(Json(convert::song_image_info(row)))
}

#[utoipa::path(
    delete,
    path = "/api/songs/{id}/images/{asset_id}",
    params(
        ("id" = Uuid, Path, description = "Song ID"),
        ("asset_id" = Uuid, Path, description = "Asset ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "songs",
    security(("session" = []))
)]
pub(crate) async fn delete_song_image(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, asset_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    if !auth.capabilities.contains(capabilities::SONGS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }

    let removed = queries::songs::unlink_image(&state.pool, id, asset_id).await?;
    if !removed {
        return Err(ApiError::NotFound);
    }
    assets::delete_if_unreferenced(&state, asset_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
