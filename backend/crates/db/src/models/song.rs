//! Song model.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A song record fetched from the database.
///
/// Does not include related entities (artists, tags, images). Use the
/// corresponding query helpers to load those via JOIN.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    /// User who created this song record. `None` if created by system.
    pub created_by: Option<Uuid>,
    pub lyrics_id: Option<Uuid>,
}

/// Input for creating a new song.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSong {
    pub title: String,
    pub created_by: Option<Uuid>,
    pub lyrics_id: Option<Uuid>,
}

/// Input for replacing a song's mutable scalar fields.
///
/// M2M relations (artists, tags, images) are updated separately
/// via `queries::songs::set_original_artists` and similar helpers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSong {
    pub title: String,
}
