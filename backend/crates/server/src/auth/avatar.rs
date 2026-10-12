//! Copies an OAuth provider avatar into local storage and links it to a user.

use sha2::{Digest, Sha256};
use uuid::Uuid;

use db::{error::DbError, models::NewInternalAsset, queries};

use crate::{assets, error::ApiError, media, state::AppState};

/// Hosts provider avatars are allowed to be downloaded from.
const ALLOWED_HOSTS: &[&str] = &["static-cdn.jtvnw.net", "cdn.discordapp.com"];

/// Largest avatar file accepted, in bytes.
pub(crate) const MAX_AVATAR_BYTES: usize = 5 * 1024 * 1024;

/// Downloads the avatar at `avatar_url`, stores it as a deduplicated image asset, and
/// links it as the user's avatar.
///
/// # Errors
///
/// Returns [`ApiError::BadRequest`] if the URL is not an allowed provider CDN URL, the
/// download fails, the file is too large, or it is not a supported image type.
pub(crate) async fn import_provider_avatar(
    state: &AppState,
    user_id: Uuid,
    avatar_url: &str,
) -> Result<(), ApiError> {
    let url = reqwest::Url::parse(avatar_url)
        .map_err(|_| ApiError::BadRequest("invalid avatar url".into()))?;
    let host_allowed = url.scheme() == "https"
        && url
            .host_str()
            .is_some_and(|host| ALLOWED_HOSTS.contains(&host));
    if !host_allowed {
        return Err(ApiError::BadRequest("avatar host not allowed".into()));
    }

    let response = state
        .http_client
        .get(url.clone())
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| ApiError::BadRequest(format!("avatar download failed: {e}")))?;

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let data = response
        .bytes()
        .await
        .map_err(|e| ApiError::BadRequest(format!("avatar download failed: {e}")))?;

    let ext = media::resolve_ext(media::MediaKind::Image, &content_type, Some(url.path()))?;
    set_user_avatar(state, user_id, &data, ext, Some(avatar_url.to_string())).await
}

/// Stores `data` as a deduplicated image asset and links it as the user's avatar.
///
/// A previous avatar asset that nothing else references is deleted along with its file.
///
/// # Errors
///
/// Returns [`ApiError::BadRequest`] if `data` exceeds [`MAX_AVATAR_BYTES`], and an
/// internal error on storage or database failures.
pub(crate) async fn set_user_avatar(
    state: &AppState,
    user_id: Uuid,
    data: &[u8],
    ext: &str,
    source_url: Option<String>,
) -> Result<(), ApiError> {
    if data.len() > MAX_AVATAR_BYTES {
        return Err(ApiError::BadRequest("avatar file too large".into()));
    }
    let hash = hex::encode(Sha256::digest(data));

    let asset = if let Some(existing) = queries::assets::get_by_hash(&state.pool, &hash).await? {
        existing
    } else {
        let saved = state.store.save("avatars", ext, data).await?;
        let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
        queries::assets::create_internal(
            &mut conn,
            &NewInternalAsset {
                title: Some("User avatar".to_string()),
                credits: None,
                source_url,
                hash,
                storage_url: saved.storage_url,
                internal_path: Some(saved.internal_path),
            },
        )
        .await?
    };

    let previous_asset_id = queries::user_avatars::get_asset_id(&state.pool, user_id).await?;
    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    queries::user_avatars::set(&mut conn, user_id, asset.id).await?;

    if let Some(previous) = previous_asset_id
        && previous != asset.id
    {
        assets::delete_if_unreferenced(state, previous).await?;
    }
    Ok(())
}
