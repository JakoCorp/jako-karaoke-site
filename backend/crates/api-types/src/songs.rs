//! Song request and response types.
//!
//! Create requests accept an optional inline `lyrics` field for convenience.
//! Updates use PUT semantics: all fields are required and missing optionals mean
//! null or remove. Lyrics are managed separately via the `/lyrics` subresource.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::common::{ArtistInfo, ImageInfo, TagInfo};
use crate::tags::SongTagKind;

/// A tag paired with its kind for application to a song.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SongTagAssignment {
    pub tag_id: Uuid,
    pub kind: SongTagKind,
}

/// Request body for `POST /api/songs`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSongRequest {
    pub title: String,
    pub artist_ids: Vec<Uuid>,
    pub tags: Vec<SongTagAssignment>,
    pub image_ids: Vec<Uuid>,
    /// Optional inline lyrics content. Creates a lyrics row in a single round trip.
    pub lyrics: Option<String>,
}

/// Request body for `PUT /api/songs/{id}`.
///
/// Lyrics are excluded, use `PUT /api/songs/{id}/lyrics` instead.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSongRequest {
    pub title: String,
    pub artist_ids: Vec<Uuid>,
    pub tags: Vec<SongTagAssignment>,
    pub image_ids: Vec<Uuid>,
}

/// Minimal song identity used when only the ID and title are needed.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SongRef {
    pub id: Uuid,
    pub title: String,
}

/// Lean song representation returned by list endpoints.
///
/// Contains enough to render a song card without a follow up request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SongSummary {
    pub id: Uuid,
    pub title: String,
    pub artists: Vec<ArtistInfo>,
}

/// Full song metadata returned by detail endpoints.
///
/// Excludes lyrics, fetch those via `GET /api/songs/{id}/lyrics` on demand.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SongResponse {
    pub id: Uuid,
    pub title: String,
    pub artists: Vec<ArtistInfo>,
    pub tags: Vec<TagInfo>,
    pub images: Vec<ImageInfo>,
}
