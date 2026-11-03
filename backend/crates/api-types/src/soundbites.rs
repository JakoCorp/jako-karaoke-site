//! Soundbite request and response types.
//!
//! Updates use PUT semantics: all fields are required and missing optionals mean
//! null. The audio is managed separately via the `/audio` subresource.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::assets::AssetInfo;
use crate::common::ArtistInfo;

/// Request body for `POST /api/soundbites`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSoundbiteRequest {
    /// Display title.
    pub title: String,
    /// Calendar date of the stream in which this soundbite occurred.
    pub soundbite_date: NaiveDate,
    /// Stream index within `soundbite_date`, 1-based. Absent when not from a stream.
    pub stream_number: Option<u8>,
    /// Soundbite index within `soundbite_date` + `stream_number`, 1-based. Absent when not from a
    /// stream.
    pub soundbite_number: Option<u16>,
    /// IDs of the artists this soundbite pertains to.
    pub artist_ids: Vec<Uuid>,
}

/// Request body for `PUT /api/soundbites/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSoundbiteRequest {
    /// Display title.
    pub title: String,
    /// Calendar date of the stream in which this soundbite occurred.
    pub soundbite_date: NaiveDate,
    /// Stream index within `soundbite_date`, 1-based. Absent when not from a stream.
    pub stream_number: Option<u8>,
    /// Soundbite index within `soundbite_date` + `stream_number`, 1-based. Absent when not from a
    /// stream.
    pub soundbite_number: Option<u16>,
    /// IDs of the artists this soundbite pertains to.
    pub artist_ids: Vec<Uuid>,
}

/// Soundbite returned by list and mutation endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SoundbiteResponse {
    /// Unique identifier.
    pub id: Uuid,
    /// Display title.
    pub title: String,
    /// Calendar date of the stream in which this soundbite occurred.
    pub soundbite_date: NaiveDate,
    /// Stream index within `soundbite_date`, 1-based. Absent when not from a stream.
    pub stream_number: Option<u8>,
    /// Soundbite index within `soundbite_date` + `stream_number`, 1-based. Absent when not from a
    /// stream.
    pub soundbite_number: Option<u16>,
    /// Artists this soundbite pertains to.
    pub artists: Vec<ArtistInfo>,
    /// The audio file of this soundbite. Absent until audio is uploaded.
    pub audio: Option<AssetInfo>,
    /// Duration in seconds, when it could be determined from the audio file.
    pub duration: Option<u32>,
}
