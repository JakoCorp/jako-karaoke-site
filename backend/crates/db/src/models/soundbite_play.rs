//! Soundbite play model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::soundbite::Soundbite;

/// A listening session of a user on a soundbite.
///
/// The session is pending while `counted_at` is `None`.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct SoundbitePlay {
    pub id: Uuid,
    pub soundbite_id: Uuid,
    pub user_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub counted_at: Option<DateTime<Utc>>,
}

/// A soundbite play session with its age measured by the database clock.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SoundbitePlayProgress {
    pub soundbite_id: Uuid,
    pub counted: bool,
    pub elapsed_seconds: i64,
}

/// A user with their counted plays on one soundbite.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SoundbitePlayerRow {
    pub user_id: Uuid,
    pub username: String,
    pub play_count: u64,
    pub last_played_at: DateTime<Utc>,
}

/// A soundbite with the counted plays of one user.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserSoundbitePlayRow {
    #[sqlx(flatten)]
    pub soundbite: Soundbite,
    pub user_play_count: u64,
    pub last_played_at: DateTime<Utc>,
}
