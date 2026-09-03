//! Performance request and response types.
//!
//! Create requests accept an optional inline `lyrics` field for convenience.
//! Updates use PUT semantics: all fields are required and missing optionals mean
//! null or remove. Lyrics are managed separately via the `/lyrics` subresource.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::common::{ArtistInfo, MediaInfo, TagInfo};
use crate::songs::{SongRef, SongSummary};
use crate::tags::PerformanceTagKind;

/// A tag paired with its kind for application to a performance.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceTagAssignment {
    pub tag_id: Uuid,
    pub kind: PerformanceTagKind,
}

/// Request body for `POST /api/performances`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePerformanceRequest {
    pub title: Option<String>,
    pub performance_date: NaiveDate,
    pub stream_number: u8,
    pub performance_number: u16,
    /// Duration in seconds.
    pub duration: Option<u32>,
    /// Offset in seconds from the start of the stream.
    pub stream_time: Option<u32>,
    pub song_ids: Vec<Uuid>,
    pub singer_ids: Vec<Uuid>,
    pub tags: Vec<PerformanceTagAssignment>,
    /// Optional inline lyrics content. Creates a lyrics row in a single round trip.
    pub lyrics: Option<String>,
}

/// Request body for `PUT /api/performances/{id}`.
///
/// Lyrics are excluded, use `PUT /api/performances/{id}/lyrics` instead.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePerformanceRequest {
    pub title: Option<String>,
    pub performance_date: NaiveDate,
    pub stream_number: u8,
    pub performance_number: u16,
    /// Duration in seconds.
    pub duration: Option<u32>,
    /// Offset in seconds from the start of the stream.
    pub stream_time: Option<u32>,
    pub song_ids: Vec<Uuid>,
    pub singer_ids: Vec<Uuid>,
    pub tags: Vec<PerformanceTagAssignment>,
}

/// Lean performance representation returned by list endpoints.
///
/// Contains enough to render a performance card without a follow up request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceSummary {
    pub id: Uuid,
    pub title: Option<String>,
    pub play_count: i32,
    /// Duration in seconds.
    pub duration: Option<u32>,
    pub performance_date: NaiveDate,
    pub stream_number: u8,
    pub performance_number: u16,
    pub singers: Vec<ArtistInfo>,
    pub songs: Vec<SongRef>,
}

/// Full performance metadata returned by detail endpoints.
///
/// Excludes lyrics, fetch those via `GET /api/performances/{id}/lyrics` on demand.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceResponse {
    pub id: Uuid,
    pub title: Option<String>,
    pub play_count: i32,
    /// Duration in seconds.
    pub duration: Option<u32>,
    /// Offset in seconds from the start of the stream.
    pub stream_time: Option<u32>,
    pub performance_date: NaiveDate,
    pub stream_number: u8,
    pub performance_number: u16,
    pub songs: Vec<SongSummary>,
    pub singers: Vec<ArtistInfo>,
    pub tags: Vec<TagInfo>,
    pub audio: Vec<MediaInfo>,
    pub video: Vec<MediaInfo>,
}
