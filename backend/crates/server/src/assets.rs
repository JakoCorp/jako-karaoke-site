//! Lifecycle helpers for stored assets shared across routes.

use db::queries;
use tracing::error;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

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
