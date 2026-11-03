//! Lifecycle helpers for stored assets shared across routes.

use db::{
    error::DbError,
    models::{Asset, NewInternalAsset},
    queries,
};
use sha2::{Digest, Sha256};
use tracing::error;
use uuid::Uuid;

use crate::{
    error::ApiError,
    media::{self, MediaKind},
    state::AppState,
};

/// An uploaded audio file with its optional attribution fields.
pub(crate) struct UploadedAudio {
    pub data: Vec<u8>,
    pub content_type: String,
    pub filename: Option<String>,
    pub title: Option<String>,
    pub credits: Option<String>,
    pub source_url: Option<String>,
}

/// Stores an uploaded audio file as an asset under `subfolder` of the file store.
///
/// Reuses the existing asset when a file with the same content hash was uploaded before.
/// Otherwise the file is saved and its playback length probed. The asset title defaults to
/// the original filename.
///
/// # Errors
///
/// Returns [`ApiError::BadRequest`] if the file is not an accepted audio format, and an
/// internal error on storage or database failures.
pub(crate) async fn save_audio(
    state: &AppState,
    subfolder: &str,
    upload: UploadedAudio,
) -> Result<Asset, ApiError> {
    let title = upload.title.or_else(|| upload.filename.clone());
    let ext = media::resolve_ext(
        MediaKind::Audio,
        &upload.content_type,
        upload.filename.as_deref(),
    )?;
    let hash = hex::encode(Sha256::digest(&upload.data));

    if let Some(existing) = queries::assets::get_by_hash(&state.pool, &hash).await? {
        return Ok(existing);
    }
    let saved = state.store.save(subfolder, ext, &upload.data).await?;
    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let asset = queries::assets::create_internal(
        &mut conn,
        &NewInternalAsset {
            title,
            credits: upload.credits,
            source_url: upload.source_url,
            hash,
            storage_url: saved.storage_url,
            internal_path: Some(saved.internal_path),
            length: media::audio_length(&upload.data),
        },
    )
    .await?;
    Ok(asset)
}

/// Deletes an asset and its stored file when nothing references it anymore.
///
/// Does nothing if the asset is still referenced or no longer exists. A failure to
/// remove the file from storage is logged and does not fail the call.
///
/// # Errors
///
/// Returns an internal error on database failures.
pub(crate) async fn delete_if_unreferenced(
    state: &AppState,
    asset_id: Uuid,
) -> Result<(), ApiError> {
    if queries::assets::reference_count(&state.pool, asset_id).await? > 0 {
        return Ok(());
    }
    let Some(asset) = queries::assets::get_by_id(&state.pool, asset_id).await? else {
        return Ok(());
    };
    queries::assets::delete(&state.pool, asset_id).await?;
    if let Some(path) = &asset.internal_path
        && let Err(e) = state.store.delete(path).await
    {
        error!("failed to delete asset file {path}: {e}");
    }
    Ok(())
}
