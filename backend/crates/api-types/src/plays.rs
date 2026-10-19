//! Performance play request and response types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::performances::PerformanceSummary;
use crate::users::UserSummary;

/// Response body for `POST /api/performances/{id}/plays`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StartPlayResponse {
    /// Identifier of the opened play session.
    pub play_id: Uuid,
    /// Seconds of listening required before the play can be completed.
    pub required_seconds: u32,
}

/// Response body for `POST /api/performances/{id}/plays/{play_id}/complete`.
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
pub struct UserPlayEntry {
    /// The performance that was played.
    #[serde(flatten)]
    pub performance: PerformanceSummary,
    /// Number of counted plays by the user on this performance.
    pub user_play_count: u64,
    /// Timestamp of the user's most recent counted play.
    pub last_played_at: DateTime<Utc>,
}
