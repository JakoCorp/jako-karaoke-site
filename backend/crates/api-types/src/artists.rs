//! Artist request and response types.
//!
//! Updates use PUT semantics: all fields are required and missing optionals mean null.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Valid kind values for an image attached to an artist.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtistImageKind {
    Avatar,
}

impl ArtistImageKind {
    /// Returns the string stored in the database for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Avatar => "avatar",
        }
    }
}

/// Valid platform kinds for an artist external link.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtistLinkKind {
    Youtube,
    Website,
    X,
    Instagram,
    Twitch,
    Other,
}

impl ArtistLinkKind {
    /// Returns the string stored in the database for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Youtube => "youtube",
            Self::Website => "website",
            Self::X => "x",
            Self::Instagram => "instagram",
            Self::Twitch => "twitch",
            Self::Other => "other",
        }
    }
}

/// An image attached to an artist with its semantic role.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistImageInfo {
    /// Unique identifier of the image record.
    pub id: Uuid,
    /// Publicly served URL for clients.
    pub public_url: String,
    /// Optional attribution text for the image creator.
    pub credits: Option<String>,
    /// Serialized [`ArtistImageKind`] value indicating the image's semantic role.
    pub kind: String,
}

/// Request body for `PATCH /api/artists/{id}/images/{image_id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateArtistImageRequest {
    /// Serialized [`ArtistImageKind`] value to assign.
    pub kind: String,
}

/// Input for attaching an existing image to an artist.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistImageInput {
    /// ID of an existing image record to attach.
    pub image_id: Uuid,
    /// Semantic role for this image.
    pub kind: ArtistImageKind,
}

/// An external link associated with an artist.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistLinkInfo {
    /// Unique identifier of the link record.
    pub id: Uuid,
    /// Fully qualified URL.
    pub url: String,
    /// Serialized [`ArtistLinkKind`] value indicating the platform.
    pub kind: String,
    /// Optional display override label.
    pub label: Option<String>,
}

/// Input for creating an external link on an artist.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistLinkInput {
    /// Fully qualified URL.
    pub url: String,
    /// Platform this link points to.
    pub kind: ArtistLinkKind,
    /// Optional display override label.
    pub label: Option<String>,
}

/// A lean artist record returned by the list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistSummary {
    /// Unique identifier.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Optional freeform bio or description.
    pub description: Option<String>,
    /// Images attached to this artist.
    pub images: Vec<ArtistImageInfo>,
    /// Number of songs for which this artist is credited as an original artist.
    pub song_count: u64,
}

/// A full artist record returned by detail and mutation endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistResponse {
    /// Unique identifier.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Optional freeform bio or description.
    pub description: Option<String>,
    /// Images attached to this artist.
    pub images: Vec<ArtistImageInfo>,
    /// External links for this artist.
    pub links: Vec<ArtistLinkInfo>,
    /// Number of songs for which this artist is credited as an original artist.
    pub song_count: u64,
    /// Number of performances in which this artist appears as a singer.
    pub performance_count: u64,
}

/// Request body for `POST /api/artists`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateArtistRequest {
    /// Display name.
    pub name: String,
    /// Optional freeform bio or description.
    pub description: Option<String>,
    /// External links to create alongside the artist.
    pub links: Vec<ArtistLinkInput>,
}

/// Request body for `PUT /api/artists/{id}`.
///
/// Images are managed via the `/images` subresource.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateArtistRequest {
    /// Display name.
    pub name: String,
    /// Optional freeform bio or description.
    pub description: Option<String>,
    /// Replaces all existing external links.
    pub links: Vec<ArtistLinkInput>,
}
