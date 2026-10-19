//! Performance play model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::performance::Performance;

/// A listening session of a user on a performance.
///
/// The session is pending while `counted_at` is `None`.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct PerformancePlay {
    pub id: Uuid,
    pub performance_id: Uuid,
    pub user_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub counted_at: Option<DateTime<Utc>>,
}

/// A play session with its age measured by the database clock.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PlayProgress {
    pub performance_id: Uuid,
    pub counted: bool,
    pub elapsed_seconds: i64,
}

/// A user with their counted plays on one performance.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PerformancePlayerRow {
    pub user_id: Uuid,
    pub username: String,
    pub play_count: u64,
    pub last_played_at: DateTime<Utc>,
}

/// A performance with the counted plays of one user.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserPlayRow {
    #[sqlx(flatten)]
    pub performance: Performance,
    pub user_play_count: u64,
    pub last_played_at: DateTime<Utc>,
}
