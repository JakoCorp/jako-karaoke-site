//! Conversions from database models to API response types.

use chrono::{DateTime, Utc};

use api_types::{
    performances::PerformanceSummary,
    playlists::{PlaylistEntry, PlaylistKind, PlaylistResponse},
    tags::TagResponse,
};
use db::models::{Tag, playlist::Playlist};

use crate::error::ApiError;

/// Converts a [`Playlist`] model to a [`PlaylistResponse`].
///
/// Returns an error if the stored kind string is not a known variant.
pub(crate) fn playlist_response(playlist: Playlist) -> Result<PlaylistResponse, ApiError> {
    let kind = match playlist.kind.as_str() {
        "user" => PlaylistKind::User,
        "official" => PlaylistKind::Official,
        "favorites" => PlaylistKind::Favorites,
        other => {
            return Err(ApiError::Internal(format!(
                "unknown playlist kind in database: {other}"
            )));
        }
    };
    Ok(PlaylistResponse {
        id: playlist.id,
        title: playlist.title,
        description: playlist.description,
        kind,
        is_public: playlist.is_public,
        created_by: playlist.created_by,
        performance_count: playlist.performance_count as u64,
    })
}

/// Wraps a [`PerformanceSummary`] with its playlist metadata.
pub(crate) fn playlist_entry(
    performance: PerformanceSummary,
    added_at: DateTime<Utc>,
) -> PlaylistEntry {
    PlaylistEntry {
        performance,
        added_at,
    }
}

/// Converts a [`Tag`] model to a [`TagResponse`].
pub(crate) fn tag_response(tag: Tag) -> TagResponse {
    TagResponse {
        id: tag.id,
        name: tag.name,
    }
}
