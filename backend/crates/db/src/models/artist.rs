//! Artist model.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An artist record fetched from the database.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Artist {
    /// Unique identifier.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Optional freeform bio or description.
    pub description: Option<String>,
    /// Number of songs for which this artist is credited as an original artist.
    pub song_count: i64,
}

/// Input for creating a new artist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewArtist {
    /// Display name.
    pub name: String,
    /// Optional freeform bio or description.
    pub description: Option<String>,
}

/// Input for replacing an artist's mutable fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateArtist {
    /// Display name.
    pub name: String,
    /// Optional freeform bio or description.
    pub description: Option<String>,
}

/// An external link associated with an artist.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct ArtistLink {
    /// Unique identifier of the link record.
    pub id: Uuid,
    /// Artist this link belongs to.
    pub artist_id: Uuid,
    /// Fully qualified URL.
    pub url: String,
    /// Platform kind stored as a string. See `ArtistLinkKind` for valid values.
    pub kind: String,
    /// Optional display override label.
    pub label: Option<String>,
}

/// Input for creating a new artist link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewArtistLink {
    /// Fully qualified URL.
    pub url: String,
    /// Platform kind stored as a string.
    pub kind: String,
    /// Optional display label.
    pub label: Option<String>,
}
