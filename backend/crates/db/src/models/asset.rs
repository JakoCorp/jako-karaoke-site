//! Asset model.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A media asset, either an internally hosted file or an externally referenced URL.
///
/// Exactly one of `(hash, storage_url)` or `external_url` is non-null per row,
/// enforced by the `chk_asset_source` database constraint.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    /// Display name or original filename.
    pub title: Option<String>,
    /// Attribution text for the content creator.
    pub credits: Option<String>,
    /// Link to the original source post or stream.
    pub source_url: Option<String>,
    /// SHA-256 hex digest of file bytes. Non-null for internal assets only.
    pub hash: Option<String>,
    /// CDN or file server URL. Non-null for internal assets only.
    pub storage_url: Option<String>,
    /// Absolute filesystem path. Non-null for internal assets only.
    pub internal_path: Option<String>,
    /// Referenced URL. Non-null for external assets only.
    pub external_url: Option<String>,
}

/// Input for creating an internally hosted asset from an uploaded file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewInternalAsset {
    pub title: Option<String>,
    pub credits: Option<String>,
    pub source_url: Option<String>,
    /// SHA-256 hex digest of the uploaded file bytes.
    pub hash: String,
    /// CDN or file server URL where the file is accessible.
    pub storage_url: String,
    /// Absolute filesystem path for the stored file.
    pub internal_path: Option<String>,
}

/// Input for creating an externally referenced asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewExternalAsset {
    pub title: Option<String>,
    pub credits: Option<String>,
    pub source_url: Option<String>,
    /// The external URL being referenced.
    pub external_url: String,
}
