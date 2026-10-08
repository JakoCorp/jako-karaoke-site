//! Shared asset response type embedded in media info structs.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Common fields describing a media asset, whether internally hosted or externally referenced.
///
/// Exactly one of `storage_url` or `external_url` is non-null per asset.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AssetInfo {
    /// Unique identifier of the asset record.
    pub asset_id: Uuid,
    /// Display name or original filename. May be absent for unnamed uploads.
    pub title: Option<String>,
    /// Attribution text for the content creator.
    pub credits: Option<String>,
    /// Link to the original source post or stream.
    pub source_url: Option<String>,
    /// CDN or file server URL. Present for internally hosted files only.
    pub storage_url: Option<String>,
    /// Referenced external URL. Present for externally linked content only.
    pub external_url: Option<String>,
    /// SHA-256 hex digest of the file bytes.
    pub hash: Option<String>,
}
