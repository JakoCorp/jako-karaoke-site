//! Play request and response types.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::performances::PerformanceSummary;
use crate::users::UserSummary;

/// A play session that was opened for a performance or a soundbite.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StartPlayResponse {
    /// Identifier of the opened play session.
    pub play_id: Uuid,
    /// Seconds of listening required before the play can be completed.
    pub required_seconds: u32,
}

/// The outcome of completing a play session.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompletePlayResponse {
    /// False when the play was valid but fell inside the cooldown of an earlier play.
    pub counted: bool,
}

/// A user with their plays on one performance.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformancePlayer {
    /// The user who played the performance.
    #[serde(flatten)]
    pub user: UserSummary,
    /// Number of counted plays by the user on this performance.
    pub play_count: u64,
    /// Timestamp of the user's most recent counted play.
    pub last_played_at: DateTime<Utc>,
}

/// A performance with the plays one user has on it.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserPerformancePlayEntry {
    /// The performance that was played.
    #[serde(flatten)]
    pub performance: PerformanceSummary,
    /// Number of counted plays by the user on this performance.
    pub user_play_count: u64,
    /// Timestamp of the user's most recent counted play.
    pub last_played_at: DateTime<Utc>,
}

/// A user with their plays on one soundbite.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SoundbitePlayer {
    /// The user who played the soundbite.
    #[serde(flatten)]
    pub user: UserSummary,
    /// Number of counted plays by the user on this soundbite.
    pub play_count: u64,
    /// Timestamp of the user's most recent counted play.
    pub last_played_at: DateTime<Utc>,
}

/// A soundbite with the plays one user has on it.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserSoundbitePlayEntry {
    /// Unique identifier of the soundbite.
    pub id: Uuid,
    /// Display title of the soundbite.
    pub title: String,
    /// Calendar date of the stream in which the soundbite occurred.
    pub soundbite_date: NaiveDate,
    /// Stream index within `soundbite_date`, 1-based. Absent when not from a stream.
    pub stream_number: Option<u8>,
    /// Soundbite index within `soundbite_date` + `stream_number`, 1-based. Absent when not from a
    /// stream.
    pub soundbite_number: Option<u16>,
    /// Number of counted plays by the user on this soundbite.
    pub user_play_count: u64,
    /// Timestamp of the user's most recent counted play.
    pub last_played_at: DateTime<Utc>,
}
