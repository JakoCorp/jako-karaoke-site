//! Performance model.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A performance record fetched from the database.
///
/// Does not include related entities (songs, singers, audio, video). Use the
/// corresponding query helpers to load those via JOIN.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Performance {
    /// Unique identifier.
    pub id: Uuid,
    /// User who created this record. `None` if created by an admin action.
    pub created_by: Option<Uuid>,
    /// Optional display title override.
    pub title: Option<String>,
    /// Performance specific lyrics override. Falls back to linked song lyrics when `None`.
    pub lyrics_id: Option<Uuid>,
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
}

/// Input for creating a new performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPerformance {
    /// User who created this record.
    pub created_by: Option<Uuid>,
    /// Optional display title.
    pub title: Option<String>,
    /// Optional performance-specific lyrics row. Falls back to the song lyrics when absent.
    pub lyrics_id: Option<Uuid>,
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
}

/// Input for replacing a performance's mutable scalar fields.
///
/// M2M relations (songs, singers) are updated separately via
/// `queries::performances::set_songs` and `set_singers`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePerformance {
    /// Optional display title.
    pub title: Option<String>,
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
}
