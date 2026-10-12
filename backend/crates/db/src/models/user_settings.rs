//! User settings model.

use serde::{Deserialize, Serialize};

/// Filename template applied when a user has not chosen their own.
pub const DEFAULT_DOWNLOAD_FILENAME_TEMPLATE: &str =
    "{date} {stream_number} #{performance_number} {title}";

/// A user's stored preferences.
///
/// A user without a row uses [`UserSettings::default`], which matches the column
/// defaults in the schema.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow, Serialize, Deserialize)]
pub struct UserSettings {
    pub download_filename_template: String,
    pub download_include_cover_art: bool,
    pub download_include_lyrics: bool,
    pub download_include_date: bool,
    pub download_include_singers: bool,
    pub download_include_original_artists: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            download_filename_template: DEFAULT_DOWNLOAD_FILENAME_TEMPLATE.to_string(),
            download_include_cover_art: true,
            download_include_lyrics: false,
            download_include_date: true,
            download_include_singers: true,
            download_include_original_artists: false,
        }
    }
}
