//! Song request and response types.
//!
//! Create requests accept an optional inline `lyrics` field for convenience.
//! Updates use PUT semantics: all fields are required and missing optionals mean
//! null or remove. Lyrics are managed separately via the `/lyrics` subresource.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::common::{ArtistInfo, TagInfo};
use crate::tags::SongTagKind;

/// Valid kind values for an image attached to a song.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SongImageKind {
    CoverArt,
}

impl SongImageKind {
    /// Returns the string stored in the database for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CoverArt => "cover_art",
        }
    }
}

/// An image attached to a song with its semantic role.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SongImageInfo {
    /// Unique identifier of the image record.
    pub id: Uuid,
    /// Publicly served URL for clients.
    pub public_url: String,
    /// Optional attribution text for the image creator.
    pub credits: Option<String>,
    /// Serialized [`SongImageKind`] value indicating the image's semantic role.
    pub kind: String,
}

/// Request body for `PATCH /api/songs/{id}/images/{image_id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSongImageRequest {
    /// Serialized [`SongImageKind`] value to assign.
    pub kind: String,
}

/// A tag paired with its kind for application to a song.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SongTagAssignment {
    /// ID of the tag to apply.
    pub tag_id: Uuid,
    /// Semantic category of this tag on the song.
    pub kind: SongTagKind,
}

/// Request body for `POST /api/songs`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSongRequest {
    /// Unique display title.
    pub title: String,
    /// IDs of original artists (composers or creators) to credit.
    pub artist_ids: Vec<Uuid>,
    /// Tags to apply at creation time.
    pub tags: Vec<SongTagAssignment>,
    /// Optional inline lyrics content. Creates a lyrics row in a single round trip.
    pub lyrics: Option<String>,
}

/// Request body for `PUT /api/songs/{id}`.
///
/// Lyrics and images are excluded, manage them via their respective subresources.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSongRequest {
    /// Unique display title.
    pub title: String,
    /// Replaces the full set of credited original artists.
    pub artist_ids: Vec<Uuid>,
    /// Replaces all existing tags.
    pub tags: Vec<SongTagAssignment>,
}

/// Minimal song identity used when only the ID and title are needed.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SongRef {
    /// Unique identifier.
    pub id: Uuid,
    /// Display title.
    pub title: String,
}

/// Lean song representation returned by list endpoints.
///
/// Contains enough to render a song card without a follow up request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SongSummary {
    /// Unique identifier.
    pub id: Uuid,
    /// Display title.
    pub title: String,
    /// Original artists credited for this song.
    pub artists: Vec<ArtistInfo>,
    /// Cover images attached to this song.
    pub images: Vec<SongImageInfo>,
    /// Number of performances that reference this song.
    pub performance_count: u64,
}

/// Full song metadata returned by detail endpoints.
///
/// Excludes lyrics, fetch those via `GET /api/songs/{id}/lyrics` on demand.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SongResponse {
    /// Unique identifier.
    pub id: Uuid,
    /// Display title.
    pub title: String,
    /// Original artists credited for this song.
    pub artists: Vec<ArtistInfo>,
    /// Tags applied to this song.
    pub tags: Vec<TagInfo>,
    /// Cover images attached to this song.
    pub images: Vec<SongImageInfo>,
    /// Number of performances that reference this song.
    pub performance_count: u64,
}
