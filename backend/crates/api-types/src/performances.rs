//! Performance request and response types.
//!
//! Create requests accept an optional inline `lyrics` field for convenience.
//! Updates use PUT semantics: all fields are required and missing optionals mean
//! null or remove. Lyrics are managed separately via the `/lyrics` subresource.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::common::{ArtistInfo, TagInfo};
use crate::songs::{SongRef, SongSummary};
use crate::tags::PerformanceTagKind;

/// Valid kind values for an audio file attached to a performance.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioKind {
    /// The canonical audio used for playback. At most one per performance.
    Primary,
    /// Any non-primary audio file, such as an alternate mix.
    Misc,
}

impl AudioKind {
    /// Returns the string stored in the database for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Misc => "misc",
        }
    }
}

/// Valid kind values for a video file attached to a performance.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum VideoKind {
    /// A short video of a single moment.
    Clip,
    /// A full stream VOD recording.
    Vod,
    /// Any other video file.
    Misc,
}

impl VideoKind {
    /// Returns the string stored in the database for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clip => "clip",
            Self::Vod => "vod",
            Self::Misc => "misc",
        }
    }
}

/// An audio file attached to a performance with its semantic role.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AudioInfo {
    /// Unique identifier of the audio record.
    pub id: Uuid,
    /// Publicly served URL for clients.
    pub public_url: String,
    /// Serialized [`AudioKind`] value.
    pub kind: String,
}

/// A video file attached to a performance with its semantic role.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VideoInfo {
    /// Unique identifier of the video record.
    pub id: Uuid,
    /// Publicly served URL for clients.
    pub public_url: String,
    /// Serialized [`VideoKind`] value.
    pub kind: String,
}

/// Request body for `PATCH /api/performances/{id}/audio/{audio_id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateAudioKindRequest {
    /// Serialized [`AudioKind`] value to assign.
    pub kind: String,
}

/// Request body for `PATCH /api/performances/{id}/video/{video_id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateVideoKindRequest {
    /// Serialized [`VideoKind`] value to assign.
    pub kind: String,
}

/// A tag paired with its kind for application to a performance.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceTagAssignment {
    /// ID of the tag to apply.
    pub tag_id: Uuid,
    /// Semantic category of this tag on the performance.
    pub kind: PerformanceTagKind,
}

/// Request body for `POST /api/performances`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePerformanceRequest {
    /// Optional display title override.
    pub title: Option<String>,
    /// Calendar date of the stream in which this performance occurred.
    pub performance_date: NaiveDate,
    /// Stream index within `performance_date`, 1-based.
    pub stream_number: u8,
    /// Performance index within `performance_date` + `stream_number`, 1-based.
    pub performance_number: u16,
    /// Duration in seconds.
    pub duration: Option<u32>,
    /// Offset in seconds from the start of the stream.
    pub stream_time: Option<u32>,
    /// IDs of songs featured in this performance.
    pub song_ids: Vec<Uuid>,
    /// IDs of artists who sang in this performance.
    pub singer_ids: Vec<Uuid>,
    /// Tags to apply at creation time.
    pub tags: Vec<PerformanceTagAssignment>,
    /// Optional inline lyrics content. Creates a lyrics row in a single round trip.
    pub lyrics: Option<String>,
}

/// Request body for `PUT /api/performances/{id}`.
///
/// Lyrics are excluded, use `PUT /api/performances/{id}/lyrics` instead.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePerformanceRequest {
    /// Optional display title override.
    pub title: Option<String>,
    /// Calendar date of the stream in which this performance occurred.
    pub performance_date: NaiveDate,
    /// Stream index within `performance_date`, 1-based.
    pub stream_number: u8,
    /// Performance index within `performance_date` + `stream_number`, 1-based.
    pub performance_number: u16,
    /// Duration in seconds.
    pub duration: Option<u32>,
    /// Offset in seconds from the start of the stream.
    pub stream_time: Option<u32>,
    /// IDs of songs featured in this performance.
    pub song_ids: Vec<Uuid>,
    /// IDs of artists who sang in this performance.
    pub singer_ids: Vec<Uuid>,
    /// Replaces all existing tags.
    pub tags: Vec<PerformanceTagAssignment>,
}

/// Lean performance representation returned by list endpoints.
///
/// Contains enough to render a performance card without a follow up request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceSummary {
    /// Unique identifier.
    pub id: Uuid,
    /// Optional display title. Absent when no explicit title has been set.
    pub title: Option<String>,
    /// Number of times this performance has been played.
    pub play_count: i32,
    /// Duration in seconds.
    pub duration: Option<u32>,
    /// Calendar date of the stream.
    pub performance_date: NaiveDate,
    /// Stream index within `performance_date`, 1-based.
    pub stream_number: u8,
    /// Performance index within `performance_date` + `stream_number`, 1-based.
    pub performance_number: u16,
    /// Artists who sang in this performance.
    pub singers: Vec<ArtistInfo>,
    /// Songs featured in this performance.
    pub songs: Vec<SongRef>,
}

/// Full performance metadata returned by detail endpoints.
///
/// Excludes lyrics, fetch those via `GET /api/performances/{id}/lyrics` on demand.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceResponse {
    /// Unique identifier.
    pub id: Uuid,
    /// Optional display title. Absent when no explicit title has been set.
    pub title: Option<String>,
    /// Number of times this performance has been played.
    pub play_count: i32,
    /// Duration in seconds.
    pub duration: Option<u32>,
    /// Offset in seconds from the start of the stream.
    pub stream_time: Option<u32>,
    /// Calendar date of the stream.
    pub performance_date: NaiveDate,
    /// Stream index within `performance_date`, 1-based.
    pub stream_number: u8,
    /// Performance index within `performance_date` + `stream_number`, 1-based.
    pub performance_number: u16,
    /// Songs featured in this performance.
    pub songs: Vec<SongSummary>,
    /// Artists who sang in this performance.
    pub singers: Vec<ArtistInfo>,
    /// Tags applied to this performance.
    pub tags: Vec<TagInfo>,
    /// Audio files attached to this performance.
    pub audio: Vec<AudioInfo>,
    /// Video files attached to this performance.
    pub video: Vec<VideoInfo>,
}
