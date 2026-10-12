//! User settings request and response types.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Preferences applied when downloading a performance to disk.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DownloadSettings {
    /// Filename template without extension. Recognized tokens are `{title}`, `{singer}`,
    /// `{original_artist}`, `{song}`, `{date}`, `{stream_number}` and `{performance_number}`.
    pub filename_template: String,
    /// Embed the song cover art in the file tags when one is hosted.
    pub include_cover_art: bool,
    /// Embed the lyrics in the file tags when they exist.
    pub include_lyrics: bool,
    /// Embed the performance date in the file tags.
    pub include_date: bool,
    /// Include the singers in the artist tag.
    pub include_singers: bool,
    /// Include the original artists of the songs in the artist tag.
    pub include_original_artists: bool,
}

/// Response body for `GET /auth/me/settings` and `PUT /auth/me/settings`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserSettingsResponse {
    /// Download preferences.
    pub download: DownloadSettings,
}

/// Request body for `PUT /auth/me/settings`. Replaces all settings.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserSettingsRequest {
    /// Download preferences.
    pub download: DownloadSettings,
}
