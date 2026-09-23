//! Image model.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An image asset stored on disk and linked to songs or artists.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Image {
    pub id: Uuid,
    /// SHA-256 hex digest of the file bytes.
    pub hash: String,
    /// Publicly served URL for clients.
    pub public_url: String,
    /// Absolute filesystem path used for actual file.
    pub internal_path: Option<String>,
    /// Optional attribution text for the image creator.
    pub credits: Option<String>,
}

/// Input for creating a new image record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewImage {
    pub hash: String,
    pub public_url: String,
    pub internal_path: Option<String>,
    /// Optional attribution text for the image creator.
    pub credits: Option<String>,
}

/// Input for replacing an image record's mutable fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateImage {
    pub public_url: String,
    pub internal_path: Option<String>,
    /// Optional attribution text for the image creator.
    pub credits: Option<String>,
}
