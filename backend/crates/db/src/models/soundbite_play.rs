//! Soundbite play model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
