//! Soundbite model.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A soundbite record fetched from the database.
///
/// Does not include related entities (artists, audio). Use the corresponding query helpers to
/// load those.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Soundbite {
    /// Unique identifier.
    pub id: Uuid,
    /// User who created this record. `None` if the user was deleted.
    pub created_by: Option<Uuid>,
    /// Display title.
    pub title: String,
    /// The audio asset of this soundbite. `None` until audio is uploaded.
    pub asset_id: Option<Uuid>,
    /// Calendar date of the stream.
    pub soundbite_date: NaiveDate,
    /// Stream index within `soundbite_date`, 1-based. `None` when not from a stream.
    pub stream_number: Option<u8>,
    /// Soundbite index within `soundbite_date` + `stream_number`, 1-based. `None` when not from a
    /// stream.
    pub soundbite_number: Option<u16>,
}

/// Input for creating a new soundbite.
///
/// The audio asset is attached afterwards via `queries::soundbites::set_asset`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSoundbite {
    /// User who created this record.
    pub created_by: Option<Uuid>,
    /// Display title.
    pub title: String,
    /// Calendar date of the stream.
    pub soundbite_date: NaiveDate,
    /// Stream index within `soundbite_date`, 1-based. `None` when not from a stream.
    pub stream_number: Option<u8>,
    /// Soundbite index within `soundbite_date` + `stream_number`, 1-based. `None` when not from a
    /// stream.
    pub soundbite_number: Option<u16>,
}

/// Input for replacing a soundbite's mutable scalar fields.
///
/// The audio asset is replaced via `queries::soundbites::set_asset` and artists via
/// `queries::soundbites::set_artists`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSoundbite {
    /// Display title.
    pub title: String,
    /// Calendar date of the stream.
    pub soundbite_date: NaiveDate,
    /// Stream index within `soundbite_date`, 1-based. `None` when not from a stream.
    pub stream_number: Option<u8>,
    /// Soundbite index within `soundbite_date` + `stream_number`, 1-based. `None` when not from a
    /// stream.
    pub soundbite_number: Option<u16>,
}
